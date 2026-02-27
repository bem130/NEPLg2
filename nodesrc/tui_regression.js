#!/usr/bin/env node
// nodesrc/tui_regression.js
// 目的:
// - WASIX TUI の回帰確認を自動化する。
// - nepl-cli で wasm を生成し、wasmer run にキー入力を送って
//   クラッシュしないこと・終了できること・基本描画が出ることを検証する。

const { spawn, spawnSync } = require('node:child_process');
const path = require('node:path');
const os = require('node:os');
const fs = require('node:fs');

function parseArgs(argv) {
    const opts = {
        input: 'examples/tui_editor/main.nepl',
        outputBase: 'tmp/editor',
        wasmer: process.env.WASMER_BIN || 'wasmer',
        neplCli: 'target/debug/nepl-cli',
        timeoutMs: 5000,
    };
    for (let i = 0; i < argv.length; i++) {
        const a = argv[i];
        if (a === '--input' && i + 1 < argv.length) opts.input = argv[++i];
        else if (a === '--output-base' && i + 1 < argv.length) opts.outputBase = argv[++i];
        else if (a === '--wasmer' && i + 1 < argv.length) opts.wasmer = argv[++i];
        else if (a === '--nepl-cli' && i + 1 < argv.length) opts.neplCli = argv[++i];
        else if (a === '--timeout-ms' && i + 1 < argv.length) opts.timeoutMs = Number(argv[++i]);
        else if (a === '-h' || a === '--help') {
            return { ...opts, help: true };
        }
    }
    return { ...opts, help: false };
}

function stripAnsi(s) {
    return String(s)
        .replace(/\x1b\[[0-9;?]*[ -/]*[@-~]/g, '')
        .replace(/\x1b\][^\x07]*(\x07|\x1b\\)/g, '');
}

function assertCond(cond, msg) {
    if (!cond) throw new Error(msg);
}

function writeFixtures(baseDir, fixtures) {
    if (!fixtures) return;
    for (const [rel, content] of Object.entries(fixtures)) {
        const p = path.join(baseDir, rel);
        fs.mkdirSync(path.dirname(p), { recursive: true });
        fs.writeFileSync(p, String(content), 'utf8');
    }
}

function assertFileEquals(baseDir, relPath, expected) {
    const p = path.join(baseDir, relPath);
    const actual = fs.readFileSync(p, 'utf8');
    if (actual !== String(expected)) {
        throw new Error(
            `file mismatch: ${relPath}\n--- expected ---\n${String(expected)}\n--- actual ---\n${actual}`
        );
    }
}

function makeVfsSnapshot() {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), 'nepl-tui-vfs-'));
    const srcTmp = path.resolve('tmp');
    const dstTmp = path.join(root, 'tmp');
    fs.mkdirSync(dstTmp, { recursive: true });
    if (fs.existsSync(srcTmp) && fs.statSync(srcTmp).isDirectory()) {
        fs.cpSync(srcTmp, dstTmp, { recursive: true });
    }
    return root;
}

function cleanupVfsSnapshot(vfsRoot) {
    try {
        fs.rmSync(vfsRoot, { recursive: true, force: true });
    } catch {}
}

function runCompile(opts) {
    const outBase = path.resolve(opts.outputBase);
    const cp = spawnSync(
        opts.neplCli,
        ['-i', opts.input, '--target', 'wasix', '--output', outBase],
        { encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 }
    );
    if (cp.status !== 0) {
        const detail = [cp.stdout || '', cp.stderr || ''].join('\n').trim();
        throw new Error(`compile failed:\n${detail}`);
    }
    return `${outBase}.wasm`;
}

function runScenario(opts, wasmPath, scenario, vfsRoot) {
    return new Promise((resolve, reject) => {
        const child = spawn(opts.wasmer, ['run', `--dir=${vfsRoot}`, wasmPath], {
            stdio: ['pipe', 'pipe', 'pipe'],
        });
        let stdout = '';
        let stderr = '';
        let done = false;

        const finish = (err, result) => {
            if (done) return;
            done = true;
            clearTimeout(killTimer);
            if (err) reject(err);
            else resolve(result);
        };

        child.stdout.on('data', (d) => {
            stdout += d.toString('utf8');
        });
        child.stderr.on('data', (d) => {
            stderr += d.toString('utf8');
        });
        child.on('error', (e) => finish(e));
        child.on('close', (code, signal) => {
            finish(null, { code, signal, stdout, stderr });
        });

        let at = 0;
        for (const step of scenario.steps) {
            at += step.delayMs;
            setTimeout(() => {
                if (done) return;
                try {
                    child.stdin.write(step.bytes, 'binary');
                } catch {}
            }, at);
        }
        setTimeout(() => {
            if (!done) {
                try { child.stdin.end(); } catch {}
            }
        }, at + 20);

        const killTimer = setTimeout(() => {
            if (done) return;
            try { child.kill('SIGKILL'); } catch {}
            finish(new Error(`scenario timeout: ${scenario.name}`));
        }, opts.timeoutMs);
    });
}

function defaultScenarios() {
    return [
        {
            name: 'smoke_quit',
            steps: [{ delayMs: 120, bytes: 'q\n' }],
            expect: (r) => {
                const merged = `${r.stdout}\n${r.stderr}`;
                assertCond(r.code === 0, `smoke_quit exit code=${r.code}`);
                assertCond(!/RuntimeError|out of bounds|call stack exhausted/i.test(merged), 'runtime error detected');
                const plain = stripAnsi(r.stdout);
                assertCond(plain.includes('NEPLg2 TUI Editor / Tab'), 'title not rendered');
                assertCond(plain.includes('Bye'), 'quit footer not rendered');
            },
        },
        {
            name: 'keys_edit_and_nav',
            steps: [
                { delayMs: 80, bytes: 'a' },
                { delayMs: 80, bytes: '\x1b[D' },
                { delayMs: 80, bytes: '\x1b[C' },
                { delayMs: 80, bytes: '\x1b[A' },
                { delayMs: 80, bytes: '\x1b[B' },
                { delayMs: 80, bytes: '1' },
                { delayMs: 80, bytes: '2' },
                { delayMs: 80, bytes: '3' },
                { delayMs: 100, bytes: 'q\n' },
            ],
            expect: (r) => {
                const merged = `${r.stdout}\n${r.stderr}`;
                assertCond(r.code === 0, `keys_edit_and_nav exit code=${r.code}`);
                assertCond(!/RuntimeError|out of bounds|call stack exhausted/i.test(merged), 'runtime error detected');
                const plain = stripAnsi(r.stdout);
                assertCond(plain.includes('Tabs: [1][2][3]'), 'tabs line missing');
                assertCond(plain.includes('Col: 2') || plain.includes('Col: 3'), 'cursor column did not move');
                assertCond(plain.includes('Bye'), 'quit footer not rendered');
            },
        },
        {
            name: 'editor_features',
            steps: [
                { delayMs: 80, bytes: '2' },
                { delayMs: 80, bytes: 'i' },
                { delayMs: 80, bytes: 'a' },
                { delayMs: 80, bytes: '\x1b' },
                { delayMs: 80, bytes: 't' },
                { delayMs: 80, bytes: 'g' },
                { delayMs: 120, bytes: 'q\n' },
            ],
            expect: (r) => {
                const merged = `${r.stdout}\n${r.stderr}`;
                assertCond(r.code === 0, `editor_features exit code=${r.code}`);
                assertCond(!/RuntimeError|out of bounds|call stack exhausted/i.test(merged), 'runtime error detected');
                const plain = stripAnsi(r.stdout);
                assertCond(plain.includes('NEPLg2 TUI Editor / Tab 2'), 'tab switch title missing');
                assertCond(plain.includes('tmp/nepl_tab2.nepl'), 'tab file path missing');
                assertCond(plain.includes('Msg: mode: INSERT'), 'insert mode message missing');
                assertCond(plain.includes('Msg: mode: NORMAL'), 'normal mode message missing');
                assertCond(plain.includes('Msg: edited'), 'edit message missing');
                assertCond(plain.includes('Msg: type:'), 'type inspect message missing');
                assertCond(plain.includes('Msg: jump:'), 'jump message missing');
                assertCond(plain.includes('Bye'), 'quit footer not rendered');
            },
        },
        {
            name: 'save_exact_tab2_insert_prefix',
            steps: [
                { delayMs: 80, bytes: '2' },
                { delayMs: 80, bytes: 'i' },
                { delayMs: 80, bytes: 'z' },
                { delayMs: 80, bytes: '\x1b' },
                { delayMs: 80, bytes: 't' },
                { delayMs: 120, bytes: 'q\n' },
            ],
            expect: (r) => {
                const merged = `${r.stdout}\n${r.stderr}`;
                assertCond(r.code === 0, `save_exact_tab2_insert_prefix exit code=${r.code}`);
                assertCond(!/RuntimeError|out of bounds|call stack exhausted/i.test(merged), 'runtime error detected');
                const plain = stripAnsi(r.stdout);
                assertCond(plain.includes('Msg: type: zfn'), 'edited source exact check failed (tab2)');
                assertCond(plain.includes('Bye'), 'quit footer not rendered');
            },
        },
        {
            name: 'save_exact_tab3_multiline',
            steps: [
                { delayMs: 80, bytes: '3' },
                { delayMs: 80, bytes: 'i' },
                { delayMs: 80, bytes: 'A' },
                { delayMs: 80, bytes: '\n' },
                { delayMs: 80, bytes: 'B' },
                { delayMs: 80, bytes: '\x1b' },
                { delayMs: 80, bytes: 't' },
                { delayMs: 120, bytes: 'q\n' },
            ],
            expect: (r) => {
                const merged = `${r.stdout}\n${r.stderr}`;
                assertCond(r.code === 0, `save_exact_tab3_multiline exit code=${r.code}`);
                assertCond(!/RuntimeError|out of bounds|call stack exhausted/i.test(merged), 'runtime error detected');
                const plain = stripAnsi(r.stdout);
                assertCond(plain.includes('Msg: type: A\nBtab3'), 'edited source exact check failed (tab3)');
                assertCond(plain.includes('Bye'), 'quit footer not rendered');
            },
        },
        {
            name: 'arrow_csi',
            steps: [
                { delayMs: 80, bytes: '\x1b[D' },
                { delayMs: 80, bytes: '\x1b[C' },
                { delayMs: 80, bytes: '\x1b[A' },
                { delayMs: 80, bytes: '\x1b[B' },
                { delayMs: 120, bytes: 'q\n' },
            ],
            expect: (r) => {
                const merged = `${r.stdout}\n${r.stderr}`;
                assertCond(r.code === 0, `arrow_csi exit code=${r.code}`);
                assertCond(!/RuntimeError|out of bounds|call stack exhausted/i.test(merged), 'runtime error detected');
                const plain = stripAnsi(r.stdout);
                assertCond(plain.includes('Bye'), 'quit footer not rendered');
            },
        },
    ];
}

async function main() {
    const opts = parseArgs(process.argv.slice(2));
    if (opts.help) {
        console.log('Usage: node nodesrc/tui_regression.js [--input <file.nepl>] [--output-base <tmp/editor>] [--wasmer <bin>] [--nepl-cli <path>] [--timeout-ms <n>]');
        process.exit(0);
    }

    const wasmPath = runCompile(opts);
    const scenarios = defaultScenarios();
    const results = [];
    for (const s of scenarios) {
        const vfsRoot = makeVfsSnapshot();
        try {
            writeFixtures(vfsRoot, s.fixtures || null);
            const r = await runScenario(opts, wasmPath, s, vfsRoot);
            s.expect(r);
            results.push({ name: s.name, exit_code: r.code, stdout_len: r.stdout.length, stderr_len: r.stderr.length });
        } finally {
            cleanupVfsSnapshot(vfsRoot);
        }
    }

    console.log(JSON.stringify({
        ok: true,
        input: opts.input,
        wasm: wasmPath,
        scenarios: results,
    }, null, 2));
}

main().catch((e) => {
    console.error(String(e && e.stack ? e.stack : e));
    process.exit(1);
});
