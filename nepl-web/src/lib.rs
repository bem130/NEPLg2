use std::collections::BTreeMap;
use std::path::PathBuf;

use nepl_core::diagnostic::{Diagnostic, Severity};
use nepl_core::error::CoreError;
use nepl_core::loader::{Loader, SourceMap};
use nepl_core::{compile_module, CompileOptions, CompileTarget};
use wasmprinter::print_bytes;
use wasm_bindgen::prelude::*;
use js_sys::{Reflect, Uint8Array};

const NEPLG2_REPO_URL: &str = "https://github.com/neknaj/NEPLg2/";
const NEPLG2_COMMIT_BASE_URL: &str = "https://github.com/neknaj/NEPLg2/commit/";

// build.rs などで NEPLG2_COMPILER_COMMIT が設定されていればそれを使う。
// wasm 実行時には git コマンド等を呼べないため、ビルド時埋め込みが前提。
const NEPLG2_COMPILER_COMMIT: &str = match option_env!("NEPLG2_COMPILER_COMMIT") {
    Some(v) if !v.is_empty() => v,
    _ => "unknown",
};

fn build_wat_header_comments() -> String {
    // WAT の行コメントは `;;` で始まる。([spec] コメントは字句要素として扱われる)
    // ここでは確実にコメント化できるよう、行ごとに `;; ` を付ける。
    let mut out = String::new();
    out.push_str(";; compiler: NEPLg2 ");
    out.push_str(NEPLG2_REPO_URL);
    out.push('\n');

    out.push_str(";; compiler commit: ");
    out.push_str(NEPLG2_COMPILER_COMMIT);
    out.push('\n');

    out.push_str(";; compiler commit url: ");
    if NEPLG2_COMPILER_COMMIT != "unknown" {
        out.push_str(NEPLG2_COMMIT_BASE_URL);
        out.push_str(NEPLG2_COMPILER_COMMIT);
    } else {
        out.push_str("(unknown)");
    }
    out.push_str("\n\n");
    out
}

fn build_attached_source_comment(entry_path: &str, source: &str) -> String {
    // 入力ソースを WAT コメントとして先頭に埋め込む（行コメントで安全に固定する）
    let mut out = String::new();
    out.push_str(";; ---- BEGIN ATTACHED SOURCE ----\n");
    out.push_str(";; path: ");
    out.push_str(entry_path);
    out.push('\n');

    for (i, line) in source.lines().enumerate() {
        // 例: ";; 0001: let x = 1"
        out.push_str(";; ");
        out.push_str(&format!("{:04}: ", i + 1));
        out.push_str(line);
        out.push('\n');
    }

    // source が末尾改行で終わっていても lines() は最後の空行を落とすため、
    // 入力の雰囲気を残したいならここで明示的に 1 行足しておく。
    if source.ends_with('\n') {
        out.push_str(";; 0000: \n");
    }

    out.push_str(";; ---- END ATTACHED SOURCE ----\n\n");
    out
}

fn decorate_wat(mut wat: String, attach_source: bool, entry_path: &str, source: &str) -> String {
    // WAT/wasmprinter の本文の前に、コンパイラ情報＋（必要なら）入力ソースを差し込む
    let mut out = String::new();
    out.push_str(&build_wat_header_comments());
    if attach_source {
        out.push_str(&build_attached_source_comment(entry_path, source));
    }
    out.push_str(&wat);
    out
}

fn make_wat(wasm: &[u8], attach_source: bool, entry_path: &str, source: &str) -> Result<String, JsValue> {
    let wat = print_bytes(wasm).map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(decorate_wat(wat, attach_source, entry_path, source))
}

fn make_wat_min(wasm: &[u8], attach_source: bool, entry_path: &str, source: &str) -> Result<String, JsValue> {
    // 先に minify してからコメントを足す（minify がコメントを削除するため）
    let wat = print_bytes(wasm).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let min = minify_wat_text(&wat);
    Ok(decorate_wat(min, attach_source, entry_path, source))
}

// main.rs の wat-min と同等の単純 minify：
// - 文字列リテラル（"..."）内はそのまま
// - 行コメント `;; ...` とブロックコメント `(; ... ;)` を除去
// - 空白を 1 個に圧縮し、括弧の前後の空白を削る
fn minify_wat_text(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;
    let mut comment_depth = 0usize;
    let mut prev_space = false;

    while let Some(c) = chars.next() {
        if in_string {
            out.push(c);
            if c == '\\' {
                // エスケープシーケンス（\" など）を 1 文字進めて保持する
                if let Some(next) = chars.next() {
                    out.push(next);
                }
                continue;
            }
            if c == '"' {
                in_string = false;
            }
            continue;
        }

        if comment_depth > 0 {
            // ネストしたブロックコメント `(; ... ;)` に対応
            if c == '(' && chars.peek() == Some(&';') {
                chars.next();
                comment_depth += 1;
                continue;
            }
            if c == ';' && chars.peek() == Some(&')') {
                chars.next();
                comment_depth = comment_depth.saturating_sub(1);
                if comment_depth == 0 && !prev_space && !out.is_empty() {
                    out.push(' ');
                    prev_space = true;
                }
                continue;
            }
            continue;
        }

        if c == '"' {
            in_string = true;
            out.push(c);
            prev_space = false;
            continue;
        }

        // 行コメント `;; ...`
        if c == ';' && chars.peek() == Some(&';') {
            chars.next();
            while let Some(next) = chars.next() {
                if next == '\n' {
                    break;
                }
            }
            if !prev_space && !out.is_empty() {
                out.push(' ');
                prev_space = true;
            }
            continue;
        }

        // ブロックコメント `(; ... ;)`
        if c == '(' && chars.peek() == Some(&';') {
            chars.next();
            comment_depth = 1;
            continue;
        }

        // 空白の圧縮
        if c.is_whitespace() {
            if !prev_space && !out.is_empty() {
                out.push(' ');
                prev_space = true;
            }
            continue;
        }

        // 括弧の直前の空白を削る
        if c == '(' {
            if out.ends_with(' ') {
                out.pop();
            }
            out.push('(');
            prev_space = false;
            continue;
        }
        if c == ')' {
            if out.ends_with(' ') {
                out.pop();
            }
            out.push(')');
            prev_space = false;
            continue;
        }

        out.push(c);
        prev_space = false;
    }

    out.trim().to_string()
}

fn parse_emit_list(emit: JsValue) -> Result<Vec<String>, JsValue> {
    // emit は "wasm"/"wat"/"wat-min" の文字列、またはそれらの配列を想定する
    if emit.is_null() || emit.is_undefined() {
        return Ok(vec!["wasm".to_string()]);
    }
    if let Some(s) = emit.as_string() {
        return Ok(vec![s]);
    }
    if js_sys::Array::is_array(&emit) {
        let arr = js_sys::Array::from(&emit);
        let mut out = Vec::with_capacity(arr.length() as usize);
        for v in arr.iter() {
            if let Some(s) = v.as_string() {
                out.push(s);
            }
        }
        if out.is_empty() {
            return Ok(vec!["wasm".to_string()]);
        }
        return Ok(out);
    }
    Err(JsValue::from_str("emit must be a string or an array of strings"))
}

fn compile_outputs_impl(
    entry_path: &str,
    source: &str,
    vfs: Option<JsValue>,
    emit: JsValue,
    attach_source: bool,
) -> Result<JsValue, JsValue> {
    // 1) wasm を生成
    let wasm = compile_wasm_with_entry(entry_path, source, vfs)
        .map_err(|msg| JsValue::from_str(&msg))?;

    // 2) 依頼された形式に応じて結果を詰める
    let emit_list = parse_emit_list(emit)?;
    let obj = js_sys::Object::new();

    for e in emit_list {
        match e.as_str() {
            "wasm" => {
                let bytes = Uint8Array::from(wasm.as_slice());
                Reflect::set(&obj, &JsValue::from_str("wasm"), &bytes.into())?;
            }
            "wat" => {
                let wat = make_wat(&wasm, attach_source, entry_path, source)?;
                Reflect::set(&obj, &JsValue::from_str("wat"), &JsValue::from_str(&wat))?;
            }
            "wat-min" => {
                let wat_min = make_wat_min(&wasm, attach_source, entry_path, source)?;
                Reflect::set(&obj, &JsValue::from_str("wat-min"), &JsValue::from_str(&wat_min))?;
            }
            other => {
                let msg = format!("unknown emit kind: {other} (expected wasm, wat, wat-min)");
                return Err(JsValue::from_str(&msg));
            }
        }
    }

    Ok(obj.into())
}

#[wasm_bindgen]
pub fn compile_source(source: &str) -> Result<Vec<u8>, JsValue> {
    compile_wasm_with_entry("/virtual/entry.nepl", source, None)
        .map_err(|msg| JsValue::from_str(&msg))
}

#[wasm_bindgen]
pub fn compile_source_with_vfs(entry_path: &str, source: &str, vfs: JsValue) -> Result<Vec<u8>, JsValue> {
    compile_wasm_with_entry(entry_path, source, Some(vfs))
        .map_err(|msg| JsValue::from_str(&msg))
}

#[wasm_bindgen]
pub fn compile_outputs(source: &str, emit: JsValue, attach_source: bool) -> Result<JsValue, JsValue> {
    // entry_path は CLI の -i 相当（lib 側では仮想パス）
    compile_outputs_impl("/virtual/entry.nepl", source, None, emit, attach_source)
}

#[wasm_bindgen]
pub fn compile_outputs_with_vfs(
    entry_path: &str,
    source: &str,
    vfs: JsValue,
    emit: JsValue,
    attach_source: bool,
) -> Result<JsValue, JsValue> {
    compile_outputs_impl(entry_path, source, Some(vfs), emit, attach_source)
}

#[wasm_bindgen]
pub fn compile_to_wat_min(source: &str, attach_source: bool) -> Result<String, JsValue> {
    let wasm = compile_wasm_with_entry("/virtual/entry.nepl", source, None)
        .map_err(|msg| JsValue::from_str(&msg))?;
    make_wat_min(&wasm, attach_source, "/virtual/entry.nepl", source)
}

#[wasm_bindgen]
pub fn compile_to_wat(source: &str) -> Result<String, JsValue> {
    let wasm = compile_wasm_with_entry("/virtual/entry.nepl", source, None)
        .map_err(|msg| JsValue::from_str(&msg))?;
    make_wat(&wasm, false, "/virtual/entry.nepl", source)
}

#[wasm_bindgen]
pub fn list_tests() -> String {
    test_sources()
        .iter()
        .map(|(name, _)| *name)
        .collect::<Vec<_>>()
        .join("\n")
}

#[wasm_bindgen]
pub fn get_stdlib_files() -> JsValue {
    let entries = stdlib_entries();
    let arr = js_sys::Array::new();
    for (path, content) in entries {
        let entry = js_sys::Array::new();
        entry.push(&JsValue::from_str(path));
        entry.push(&JsValue::from_str(content));
        arr.push(&entry);
    }
    arr.into()
}

#[wasm_bindgen]
pub fn get_example_files() -> JsValue {
    let entries = example_entries();
    let arr = js_sys::Array::new();
    for (path, content) in entries {
        let entry = js_sys::Array::new();
        entry.push(&JsValue::from_str(path));
        entry.push(&JsValue::from_str(content));
        arr.push(&entry);
    }
    arr.into()
}

#[wasm_bindgen]
pub fn get_readme() -> String {
    readme_content().to_string()
}

#[wasm_bindgen]
pub fn compile_test(name: &str) -> Result<Vec<u8>, JsValue> {
    let src = test_sources()
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, src)| *src)
        .ok_or_else(|| JsValue::from_str("unknown test"))?;
    compile_wasm_with_entry(&format!("/virtual/tests/{name}.nepl"), src, None)
        .map_err(|msg| JsValue::from_str(&msg))
}

fn compile_wasm_with_entry(entry_path: &str, source: &str, vfs: Option<JsValue>) -> Result<Vec<u8>, String> {
    let stdlib_root = PathBuf::from("/stdlib");
    let mut sources = stdlib_sources(&stdlib_root);
    
    // VFS ファイルが渡された場合は sources にマージする
    if let Some(vfs_val) = vfs {
        if vfs_val.is_object() {
            let entries = js_sys::Object::entries(&vfs_val.into());
            for entry in entries.iter() {
                let pair = js_sys::Array::from(&entry);
                let path_str = pair.get(0).as_string().unwrap_or_default();
                let content = pair.get(1).as_string().unwrap_or_default();
                if !path_str.is_empty() {
                    sources.insert(PathBuf::from(path_str), content);
                }
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&format!("Loader context contains {} files", sources.len()).into());
    
    let mut loader = Loader::new(stdlib_root);
    let mut provider = |path: &PathBuf| {
        sources
            .get(path)
            .cloned()
            .ok_or_else(|| {
                let msg = format!(
                    "missing source: {}. Available sources: {:?}",
                    path.display(),
                    sources.keys().collect::<Vec<_>>()
                );
                #[cfg(target_arch = "wasm32")]
                web_sys::console::error_1(&msg.clone().into());
                nepl_core::loader::LoaderError::Io(msg)
            })
    };
    let loaded = loader
        .load_inline_with_provider(PathBuf::from(entry_path), source.to_string(), &mut provider)
        .map_err(|e| e.to_string())?;
    let artifact = compile_module(
        loaded.module,
        CompileOptions {
            target: Some(CompileTarget::Wasi),
            verbose: false,
            profile: None,
        },
    )
    .map_err(|e| render_core_error(e, &loaded.source_map))?;
    Ok(artifact.wasm)
}

fn render_core_error(err: CoreError, sm: &SourceMap) -> String {
    match err {
        CoreError::Diagnostics(diags) => render_diagnostics(&diags, sm),
        other => other.to_string(),
    }
}

fn render_diagnostics(diags: &[Diagnostic], sm: &SourceMap) -> String {
    let mut out = String::new();
    const RESET: &str = "\x1b[0m";
    const BOLD: &str = "\x1b[1m";
    const RED: &str = "\x1b[31m";
    const YELLOW: &str = "\x1b[33m";
    const CYAN: &str = "\x1b[36m";
    const BLUE: &str = "\x1b[34m";

    for d in diags {
        let (severity_str, severity_color) = match d.severity {
            Severity::Error => ("error", RED),
            Severity::Warning => ("warning", YELLOW),
        };
        let code = d.code.unwrap_or("");
        let primary = &d.primary;
        let (line, col) = sm
            .line_col(primary.span.file_id, primary.span.start)
            .unwrap_or((0, 0));
        let path = sm
            .path(primary.span.file_id)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<unknown>".into());
        let code_display = if code.is_empty() {
            String::new()
        } else {
            format!("[{code}]")
        };
        
        // エラーヘッダ
        out.push_str(&format!(
            "{color}{bold}{sev}{code_disp}{reset}: {bold}{message}{reset}\n",
            color = severity_color,
            bold = BOLD,
            sev = severity_str,
            code_disp = code_display,
            reset = RESET,
            message = d.message
        ));
        
        // 位置ポインタ
        out.push_str(&format!(
            " {blue}-->{reset} {path}:{line}:{col}\n",
            blue = BLUE,
            reset = RESET,
            path = path,
            line = line + 1,
            col = col + 1
        ));

        if let Some(line_str) = sm.line_str(primary.span.file_id, line) {
            out.push_str(&format!(
                "  {blue}{line_num:>4} |{reset} {text}\n",
                blue = BLUE,
                reset = RESET,
                line_num = line + 1,
                text = line_str
            ));
            let caret_pos = col;
            out.push_str(&format!(
                "       {blue}|{reset} {spaces}{color}{bold}{carets}{reset}\n",
                blue = BLUE,
                reset = RESET,
                spaces = " ".repeat(caret_pos),
                color = severity_color,
                bold = BOLD,
                carets = "^".repeat(primary.span.len().max(1) as usize)
            ));
        }
        for label in &d.secondary {
            let (l, c) = sm
                .line_col(label.span.file_id, label.span.start)
                .unwrap_or((0, 0));
            let p = sm
                .path(label.span.file_id)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "<unknown>".into());
            let msg = label.message.as_ref().map(|m| m.as_str()).unwrap_or("");
            out.push_str(&format!(
                " {blue}note:{reset} {p}:{line}:{col}: {msg}\n",
                blue = BLUE,
                reset = RESET,
                line = l + 1,
                col = c + 1
            ));
        }
        out.push('\n');
    }
    out
}

fn stdlib_sources(root: &PathBuf) -> BTreeMap<PathBuf, String> {
    let mut map = BTreeMap::new();
    for (path, src) in stdlib_entries() {
        map.insert(root.join(path), src.to_string());
    }
    map
}

include!(concat!(env!("OUT_DIR"), "/stdlib_entries.rs"));

fn stdlib_entries() -> &'static [(&'static str, &'static str)] {
    STD_LIB_ENTRIES
}

fn example_entries() -> &'static [(&'static str, &'static str)] {
    EXAMPLE_ENTRIES
}

fn readme_content() -> &'static str {
    README_CONTENT
}

fn test_sources() -> &'static [(&'static str, &'static str)] {
    TEST_ENTRIES
}
