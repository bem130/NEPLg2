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

async function loadCompiler(compilerDir) {
    const dir = compilerDir || process.cwd();
    const jsName = fs.readdirSync(dir).find(f => /^nepl-web-.*\.js$/.test(f));
    const wasmName = fs.readdirSync(dir).find(f => /^nepl-web-.*_bg\.wasm$/.test(f));
    if (!jsName || !wasmName) {
        throw new Error(`compiler not found in ${dir} (need nepl-web-*.js and *_bg.wasm)`);
    }
    const jsPath = path.resolve(dir, jsName);
    const wasmPath = path.resolve(dir, wasmName);

    const mod = await import(pathToFileUrl(jsPath));
    const wasmBytes = fs.readFileSync(wasmPath);
    mod.initSync({ module: wasmBytes });
    return mod;
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

async function runOneTest(mod, t) {
    const attrs = new Set(t.attrs || []);
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

    let wasmBytes;
    try {
        const out = mod.compile_outputs(source, ["wasm"], false);
        wasmBytes = out.wasm;
    } catch (e) {
        return { ...t, ok: false, reason: "compile error", detail: String(e) };
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
