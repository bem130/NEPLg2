#!/usr/bin/env node
/**
 * tests.js
 *
 * /tests/*.n.md の doctest を一括実行し、結果(JSON)を -o で指定したパスへ書き出す。
 *
 * 例:
 *   node nodesrc/tests.js -o out.json
 *   node nodesrc/tests.js --tests-dir ./tests --compiler-dir . --jobs 4 -o out.json
 *
 * 注意:
 * - このツールは .n.md のみ対象（.nepl は対象外）。
 * - 実行は nodesrc/run_test.js の runAll を利用する。
 */

const fs = require("fs");
const path = require("path");
const { runAll } = require("./run_test.js");

function parseArgs(argv) {
    let outPath = null;
    let testsDir = path.resolve(process.cwd(), "tests");
    let jobs = null;
    // 既定は dist（trunk build の出力）を優先する
    let compilerDir = fs.existsSync(path.resolve(process.cwd(), "dist"))
        ? path.resolve(process.cwd(), "dist")
        : process.cwd();

    let i = 2;
    while (i < argv.length) {
        const a = argv[i];
        if (a === "-o") {
            outPath = argv[i + 1];
            i += 2;
            continue;
        }
        if (a === "--tests-dir") {
            testsDir = path.resolve(argv[i + 1] || testsDir);
            i += 2;
            continue;
        }
        if (a === "--jobs") {
            jobs = parseInt(argv[i + 1] || "1", 10);
            i += 2;
            continue;
        }
        if (a === "--compiler-dir") {
            compilerDir = path.resolve(argv[i + 1] || compilerDir);
            i += 2;
            continue;
        }
        throw new Error(`unknown arg: ${a}`);
    }
    if (!outPath) throw new Error("-o out.json is required");
    return { outPath: path.resolve(outPath), testsDir, jobs, compilerDir };
}

function listNmdFiles(dir) {
    if (!fs.existsSync(dir)) return [];
    const entries = fs.readdirSync(dir, { withFileTypes: true });
    const out = [];
    for (const e of entries) {
        if (!e.isFile()) continue;
        if (e.name.endsWith(".n.md")) out.push(path.join(dir, e.name));
    }
    out.sort();
    return out;
}

function ensureDirForFile(p) {
    fs.mkdirSync(path.dirname(p), { recursive: true });
}

async function main() {
    const { outPath, testsDir, jobs, compilerDir } = parseArgs(process.argv);
    const inputs = listNmdFiles(testsDir);
    if (inputs.length === 0) {
        console.error(`no .n.md in ${testsDir}`);
        process.exit(2);
    }
    const res = await runAll({ inputs, compilerDir, jobs });
    ensureDirForFile(outPath);
    fs.writeFileSync(outPath, JSON.stringify(res, null, 2), "utf-8");
    if (res.summary.failed > 0) process.exitCode = 1;
}

main().catch((e) => {
    console.error(String(e && e.stack ? e.stack : e));
    process.exit(1);
});
