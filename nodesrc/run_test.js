/**
 * run_test.js
 *
 * Node.js 版 doctest ランナー（最小機能）。
 *
 * 目的:
 * - .nepl（//: doc comment）と .n.md（文書中）に埋め込まれた neplg2:test を抽出し、
 *   nepl-web-*.js + *_bg.wasm（wasm-bindgen コンパイラ）でコンパイルして実行する。
 *
 * サポート（最小）:
 * - compile_fail / should_panic
 * - stdin: "...", stdin: ./file
 * - stdout: "...", stdout: ./file, stdout: mlstr:
 * - normalize_newlines / strip_ansi
 * - 並列実行（worker_threads）: --jobs N を cli.js から渡す
 *
 * 出力:
 * - { summary: {total, passed, failed}, results: [...] }
 *
 * 実装上の注意:
 * - normalize_newlines が有効なときは、実運用上の差分ノイズを減らすため、
 *   末尾の単一改行だけは「等価」とみなす（stdout の末尾 \n の有無差を吸収する）。
 */

const fs = require("fs");
const path = require("path");
const os = require("os");
const { Worker, isMainThread, parentPort, workerData } = require("worker_threads");

function stripAnsi(s) {
    return s.replace(/\x1b\[[0-9;]*m/g, "");
}

function normalizeNewlines(s) {
    return s.replace(/\r\n/g, "\n").replace(/\r/g, "\n");
}

function trimOneTrailingNewline(s) {
    return s.endsWith("\n") ? s.slice(0, -1) : s;
}

function parseAttrs(attrText) {
    const set = new Set();
    if (!attrText) return set;
    for (const part of attrText.split(",")) {
        const t = part.trim();
        if (t) set.add(t);
    }
    return set;
}

function parseQuotedOrPath(value, baseDir) {
    const v = value.trim();
    if (v.startsWith('"')) {
        try {
            return JSON.parse(v);
        } catch {
            return v.slice(1, v.endsWith('"') ? -1 : undefined);
        }
    }
    const p = path.resolve(baseDir, v);
    if (fs.existsSync(p) && fs.statSync(p).isFile()) {
        return fs.readFileSync(p, "utf-8");
    }
    return v;
}

function parseRetSpec(value) {
    const v = (value || "").trim();
    if (!v) return null;

    // 文字列は JSON 文字列（"..."）として受け取る
    if (v.startsWith('"')) {
        // stdin/stdout と同様に JSON.parse でエスケープを解釈する
        try {
            return { kind: "str", value: JSON.parse(v) };
        } catch {
            return { kind: "str", value: v.slice(1, v.endsWith('"') ? -1 : undefined) };
        }
    }

    const low = v.toLowerCase();
    if (low === "nan") return { kind: "f64", value: NaN };
    if (low === "inf" || low === "+inf" || low === "infinity" || low === "+infinity") return { kind: "f64", value: Infinity };
    if (low === "-inf" || low === "-infinity") return { kind: "f64", value: -Infinity };

    // 浮動小数点判定（'.' / 'e' / 'E' を含む）
    const isFloat = /[.eE]/.test(v);
    const num = Number(v);
    if (Number.isNaN(num)) return null;

    return { kind: isFloat ? "f64" : "i32", value: num };
}

function readNeplStrFromMemory(mem, ptr) {
    if (!mem || !mem.buffer) return null;
    const u8 = new Uint8Array(mem.buffer);
    const off = ptr >>> 0;
    if (off + 4 > u8.length) return null;
    const len = (u8[off]) | (u8[off + 1] << 8) | (u8[off + 2] << 16) | (u8[off + 3] << 24);
    const start = off + 4;
    const end = start + (len >>> 0);
    if (end > u8.length) return null;
    try {
        return new TextDecoder("utf-8", { fatal: false }).decode(u8.slice(start, end));
    } catch {
        return Buffer.from(u8.slice(start, end)).toString("utf-8");
    }
}

function floatEq(a, b) {
    if (Number.isNaN(a) && Number.isNaN(b)) return true;
    if (!Number.isFinite(a) || !Number.isFinite(b)) return a === b;
    const diff = Math.abs(a - b);
    const scale = Math.max(1.0, Math.abs(b));
    return diff <= 1e-9 * scale;
}

function compareRet(got, mem, spec) {
    if (!spec) return true;
    if (spec.kind === "i32") return (got | 0) === (spec.value | 0);
    if (spec.kind === "f64") return floatEq(Number(got), Number(spec.value));
    if (spec.kind === "str") {
        const s = readNeplStrFromMemory(mem, got | 0);
        return s === spec.value;
    }
    return false;
}

function parseMlstr(lines, startIndex, stripPrefix) {
    let i = startIndex;
    const out = [];
    while (i < lines.length) {
        const raw = lines[i];
        const s = stripPrefix(raw);
        const m = /^\s*##:\s?(.*)$/.exec(s);
        if (!m) break;
        out.push(m[1]);
        i += 1;
    }
    return { text: out.join("\n"), nextIndex: i };
}

function extractTestsFromNepl(filePath) {
    const src = fs.readFileSync(filePath, "utf-8");
    const baseDir = path.dirname(filePath);
    const lines = normalizeNewlines(src).split("\n");

    const tests = [];
    let i = 0;

    const stripPrefix = (ln) => {
        if (!ln.startsWith("//:")) return null;
        return ln.slice(3).trimEnd();
    };

    while (i < lines.length) {
        const b = stripPrefix(lines[i]);
        if (b === null) break; // 先頭 doc block のみ対象（現段階）

        const m = /^\s*neplg2:test(?:\[(.*)\])?\s*$/.exec(b);
        if (m) {
            const attrs = parseAttrs(m[1] || "");
            const t = {
                origin: "nepl",
                filePath,
                name: `nepl:${path.basename(filePath)}:${tests.length}`,
                attrs: Array.from(attrs),
                stdin: null,
                stdout: null,
                stderr: null,
                retSpec: null,
                code: null, // ファイル全体
            };

            i += 1;
            while (i < lines.length) {
                const b2 = stripPrefix(lines[i]);
                if (b2 === null) break;

                // 次の test が来たらここで打ち切る（次ループで新しい test として処理）
                if (/^\s*neplg2:test(?:\[.*\])?\s*$/.test(b2.trim())) break;

                const kv = /^(\s*[a-zA-Z_]+)\s*:\s*(.*)$/.exec(b2);
                if (kv) {
                    const key = kv[1].trim();
                    const val = kv[2] || "";
                    if (key === "stdin") t.stdin = parseQuotedOrPath(val, baseDir);
                    if (key === "stdout") {
                        if (val.trim() === "mlstr:") {
                            const ml = parseMlstr(lines, i + 1, (ln) => stripPrefix(ln) ?? "");
                            t.stdout = ml.text;
                            i = ml.nextIndex;
                            continue;
                        } else {
                            t.stdout = parseQuotedOrPath(val, baseDir);
                        }
                    }
                    if (key === "stderr") t.stderr = parseQuotedOrPath(val, baseDir);
                if (key === "ret" || key === "result") t.retSpec = parseRetSpec(val);
                }
                i += 1;
            }

            tests.push(t);
            continue;
        }

        i += 1;
    }

    return tests;
}

function extractTestsFromNmd(filePath) {
    const src = fs.readFileSync(filePath, "utf-8");
    const baseDir = path.dirname(filePath);
    const lines = normalizeNewlines(src).split("\n");

    const tests = [];
    let i = 0;

    while (i < lines.length) {
        const ln = lines[i].trimEnd();
        const m = /^neplg2:test(?:\[(.*)\])?\s*$/.exec(ln);
        if (!m) {
            i += 1;
            continue;
        }

        const attrs = parseAttrs(m[1] || "");
        const t = {
            origin: "nmd",
            filePath,
            name: `nmd:${path.basename(filePath)}:${tests.length}`,
            attrs: Array.from(attrs),
            stdin: null,
            stdout: null,
            stderr: null,
            retSpec: null,
            code: "",
        };
        i += 1;

        // メタ行: 次の ``` まで
        while (i < lines.length && !/^```/.test(lines[i])) {
            const b = lines[i].trimEnd();
            const kv = /^(\s*[a-zA-Z_]+)\s*:\s*(.*)$/.exec(b);
            if (kv) {
                const key = kv[1].trim();
                const val = kv[2] || "";
                if (key === "stdin") t.stdin = parseQuotedOrPath(val, baseDir);
                if (key === "stdout") {
                    if (val.trim() === "mlstr:") {
                        const ml = parseMlstr(lines, i + 1, (x) => x);
                        t.stdout = ml.text;
                        i = ml.nextIndex;
                        continue;
                    } else {
                        t.stdout = parseQuotedOrPath(val, baseDir);
                    }
                }
                if (key === "stderr") t.stderr = parseQuotedOrPath(val, baseDir);
                if (key === "ret" || key === "result") t.retSpec = parseRetSpec(val);
                if (key === "ret" || key === "result") {
                    t.retSpec = parseRetSpec(val);
                }
            }
            i += 1;
        }

        // フェンス
        const f = /^```([^\s`]*)\s*$/.exec(lines[i] || "");
        if (!f) throw new Error(`doctest missing fence in ${filePath} around line ${i + 1}`);
        i += 1;

        const codeLines = [];
        while (i < lines.length && !/^```/.test(lines[i])) {
            let cl = lines[i];
            // 表示隠し行: //:| を剥がす（テストには含める）
            cl = cl.replace(/^\s*\/\/:\|\s?/, "");
            codeLines.push(cl);
            i += 1;
        }
        if (i < lines.length && /^```/.test(lines[i])) i += 1;

        t.code = codeLines.join("\n");
        tests.push(t);
    }

    return tests;
}

function pathToFileUrl(p) {
    let x = path.resolve(p).replace(/\\/g, "/");
    if (!x.startsWith("/")) x = "/" + x;
    return "file://" + x;
}

function uniq(list) {
    const out = [];
    const seen = new Set();
    for (const x of list) {
        const k = String(x);
        if (seen.has(k)) continue;
        seen.add(k);
        out.push(x);
    }
    return out;
}

function isDir(p) {
    try {
        return fs.statSync(p).isDirectory();
    } catch {
        return false;
    }
}

function findCompilerInDir(dir) {
    if (!isDir(dir)) return null;
    let files;
    try {
        files = fs.readdirSync(dir);
    } catch {
        return null;
    }

    const jsList = files.filter(f => /^nepl-web-.*\.js$/.test(f));
    const wasmList = files.filter(f => /^nepl-web-.*_bg\.wasm$/.test(f));
    if (jsList.length === 0 || wasmList.length === 0) return null;

    // 同一プレフィックス（nepl-web-<hash>.js と nepl-web-<hash>_bg.wasm）の組を優先
    const wasmSet = new Set(wasmList);
    for (const jsName of jsList) {
        const base = jsName.replace(/\.js$/, "");
        const candWasm = base + "_bg.wasm";
        if (wasmSet.has(candWasm)) {
            return { dir, jsName, wasmName: candWasm };
        }
    }
    // それでも見つからない場合は先頭を採用
    return { dir, jsName: jsList[0], wasmName: wasmList[0] };
}

function candidateCompilerDirs(compilerDir) {
    const cwd = process.cwd();
    const here = __dirname;

    const base = [];
    if (compilerDir) {
        const r = path.resolve(compilerDir);
        base.push(r);
        // 指定がリポジトリルート（またはその近傍）でも動くようにサブディレクトリも試す
        base.push(path.join(r, "dist"));
        base.push(path.join(r, "web", "dist"));

        // 指定が dist / web/dist の場合は相互の候補も追加
        if (path.basename(r) === "dist") {
            base.push(path.join(path.dirname(r), "web", "dist"));
        }
        if (path.basename(r) === "web" && path.basename(path.dirname(r)) !== "") {
            base.push(path.join(path.dirname(r), "dist"));
        }
        if (r.endsWith(path.join("web", "dist"))) {
            base.push(path.join(path.dirname(path.dirname(r)), "dist"));
        }
    }

    // 実行場所が repo ルート / web / nodesrc のいずれでも拾えるように候補を列挙
    base.push(path.join(cwd, "dist"));
    base.push(path.join(cwd, "web", "dist"));
    base.push(path.join(cwd, "..", "dist"));
    base.push(path.join(cwd, "..", "web", "dist"));

    // nodesrc 直下で実行した場合（__dirname = .../nodesrc）を考慮
    base.push(path.join(here, "..", "dist"));
    base.push(path.join(here, "..", "web", "dist"));
    base.push(path.join(here, "..", "..", "dist"));
    base.push(path.join(here, "..", "..", "web", "dist"));

    return uniq(base.map(p => path.resolve(p)));
}

async function loadCompiler(compilerDir) {
    const tried = [];
    for (const dir of candidateCompilerDirs(compilerDir)) {
        tried.push(dir);
        const found = findCompilerInDir(dir);
        if (!found) continue;
        const jsPath = path.resolve(found.dir, found.jsName);
        const wasmPath = path.resolve(found.dir, found.wasmName);

        const mod = await import(pathToFileUrl(jsPath));
        const wasmBytes = fs.readFileSync(wasmPath);
        mod.initSync({ module: wasmBytes });
        return mod;
    }

    // エラーメッセージに「どこを探したか」を含める（デバッグ用）
    const head = compilerDir ? `compiler not found for compilerDir=${compilerDir}` : "compiler not found";
    throw new Error(
        `${head}. Need nepl-web-*.js and nepl-web-*_bg.wasm. Tried:\n` + tried.map(x => `- ${x}`).join("\n")
    );
}

function runWasiBytes(wasmBytes, stdinStr) {
    const { WASI } = require("wasi");
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "nmtest-"));
    const inPath = path.join(tmpDir, "stdin.txt");
    const outPath = path.join(tmpDir, "stdout.txt");
    const errPath = path.join(tmpDir, "stderr.txt");
    fs.writeFileSync(inPath, stdinStr || "", "utf-8");
    const stdinFd = fs.openSync(inPath, "r");
    const stdoutFd = fs.openSync(outPath, "w");
    const stderrFd = fs.openSync(errPath, "w");

    const wasi = new WASI({
        version: "preview1",
        args: ["prog.wasm"],
        env: {},
        preopens: {},
        stdin: stdinFd,
        stdout: stdoutFd,
        stderr: stderrFd,
    });

    let exitCode = 0;
    let trapped = null;
    try {
        const mod = new WebAssembly.Module(wasmBytes);
        const inst = new WebAssembly.Instance(mod, wasi.getImportObject());
        wasi.start(inst);
    } catch (e) {
        trapped = e;
        const msg = String(e && e.message ? e.message : e);
        const m = /WASIExitError: wasi exited with exit code: (\d+)/.exec(msg);
        if (m) exitCode = parseInt(m[1], 10);
        else exitCode = 1;
    } finally {
        fs.closeSync(stdinFd);
        fs.closeSync(stdoutFd);
        fs.closeSync(stderrFd);
    }
    const stdout = fs.readFileSync(outPath, "utf-8");
    const stderr = fs.readFileSync(errPath, "utf-8");
    try { fs.rmSync(tmpDir, { recursive: true, force: true }); } catch {}
    return { exitCode, stdout, stderr, trapped: trapped ? String(trapped) : null };
}


function instantiateAndCallMain(wasmBytes) {
    const mod = new WebAssembly.Module(wasmBytes);
    const fallbacks = [
        {},
        { env: {} },
        { wasi_snapshot_preview1: {} },
        { env: {}, wasi_snapshot_preview1: {} },
    ];
    let lastErr = null;
    for (const imp of fallbacks) {
        try {
            const inst = new WebAssembly.Instance(mod, imp);
            const f = inst.exports && inst.exports.main;
            if (typeof f !== "function") {
                return { ok: false, reason: "no exported main", value: null, memory: null };
            }
            const v = f();
            const mem = inst.exports && inst.exports.memory;
            return { ok: true, value: v, memory: mem && mem.buffer ? mem : null };
        } catch (e) {
            lastErr = e;
        }
    }
    return { ok: false, reason: "instantiate failed", value: null, memory: null, detail: String(lastErr) };
}


async function runOneTest(mod, t) {
    const attrs = new Set(t.attrs || []);
    if (attrs.has("skip")) {
        return { ...t, ok: true, reason: "skipped" };
    }
    const norm = (s) => {
        let x = s == null ? "" : String(s);
        if (attrs.has("strip_ansi")) x = stripAnsi(x);
        if (attrs.has("normalize_newlines")) x = normalizeNewlines(x);
        if (attrs.has("normalize_newlines")) x = trimOneTrailingNewline(x);
        return x;
    };

    let source = t.code;
    if (t.origin === "nepl") {
        source = fs.readFileSync(t.filePath, "utf-8");
    }

    if (attrs.has("compile_fail")) {
        try {
            mod.compile_outputs(source, ["wasm"], false);
            return { ...t, ok: false, reason: "expected compile_fail but compiled", detail: null };
        } catch (e) {
            return { ...t, ok: true, reason: "compile_fail ok", detail: String(e) };
        }
    }

    if (attrs.has("compile_ok")) {
        try {
            mod.compile_outputs(source, ["wasm"], false);
            return { ...t, ok: true, reason: "compile_ok ok", detail: null };
        } catch (e) {
            return { ...t, ok: false, reason: "expected compile_ok but compile error", detail: String(e) };
        }
    }


    let wasmBytes;
    try {
        const out = mod.compile_outputs(source, ["wasm"], false);
        wasmBytes = out.wasm;
    } catch (e) {
        return { ...t, ok: false, reason: "compile error", detail: String(e) };
    }


    if (t.retSpec != null) {
        const call = instantiateAndCallMain(wasmBytes);
        if (!call.ok) {
            return { ...t, ok: false, reason: call.reason, detail: call.detail || null };
        }
        const got = (typeof call.value === "bigint") ? Number(call.value) : call.value;
        if (!compareRet(got, call.memory, t.retSpec)) {
            return { ...t, ok: false, reason: "ret mismatch", expected: { ret: t.retSpec }, actual: { ret: got } };
        }
        return { ...t, ok: true, reason: "ret ok", actual: { ret: got } };
    }

    const res = runWasiBytes(wasmBytes, t.stdin || "");
    const outStdout = norm(res.stdout);
    const outStderr = norm(res.stderr);
    const expStdout = t.stdout != null ? norm(t.stdout) : null;
    const expStderr = t.stderr != null ? norm(t.stderr) : null;

    if (attrs.has("should_panic")) {
        if (res.exitCode !== 0) return { ...t, ok: true, reason: "should_panic ok", runtime: res };
        return { ...t, ok: false, reason: "expected panic but exit_code=0", runtime: res };
    }

    if (expStdout != null && outStdout !== expStdout) {
        return { ...t, ok: false, reason: "stdout mismatch", runtime: res, expected: { stdout: expStdout }, actual: { stdout: outStdout } };
    }
    if (expStderr != null && outStderr !== expStderr) {
        return { ...t, ok: false, reason: "stderr mismatch", runtime: res, expected: { stderr: expStderr }, actual: { stderr: outStderr } };
    }
    if (expStdout == null && expStderr == null) {
        if (res.exitCode !== 0) return { ...t, ok: false, reason: "nonzero exit without expectation", runtime: res };
    }
    return { ...t, ok: true, reason: "ok", runtime: res };
}

async function runTestsInWorker(payload) {
    const mod = await loadCompiler(payload.compilerDir);
    const results = [];
    for (const t of payload.tests) results.push(await runOneTest(mod, t));
    return results;
}

function summarize(results) {
    const total = results.length;
    const passed = results.filter(r => r.ok).length;
    const failed = total - passed;
    return { total, passed, failed };
}

async function runAll({ inputs, compilerDir, jobs }) {
    let tests = [];
    for (const inp of inputs) {
        if (inp.endsWith(".nepl")) tests = tests.concat(extractTestsFromNepl(inp));
        else if (inp.endsWith(".n.md")) tests = tests.concat(extractTestsFromNmd(inp));
    }

    const j = Math.max(1, Math.min(jobs || os.cpus().length, tests.length || 1));
    if (j === 1 || tests.length <= 1) {
        const mod = await loadCompiler(compilerDir);
        const results = [];
        for (const t of tests) results.push(await runOneTest(mod, t));
        return { summary: summarize(results), results };
    }

    const chunks = [];
    for (let k = 0; k < j; k++) chunks.push([]);
    for (let idx = 0; idx < tests.length; idx++) chunks[idx % j].push(tests[idx]);

    const workers = chunks.map((chunk) => new Promise((resolve, reject) => {
        const w = new Worker(__filename, { workerData: { compilerDir, tests: chunk } });
        w.on("message", resolve);
        w.on("error", reject);
        w.on("exit", (code) => { if (code !== 0) reject(new Error(`worker exit ${code}`)); });
    }));

    const parts = await Promise.all(workers);
    const results = parts.flat();
    return { summary: summarize(results), results };
}

if (!isMainThread) {
    runTestsInWorker(workerData)
        .then((res) => parentPort.postMessage(res))
        .catch((e) => parentPort.postMessage([{ ok: false, reason: "worker_error", detail: String(e) }]));
}

module.exports = {
    extractTestsFromNepl,
    extractTestsFromNmd,
    runAll,
};
