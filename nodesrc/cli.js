#!/usr/bin/env node
/**
 * cli.js
 *
 * HTML生成を最優先としつつ、-o test=out.json が指定されていれば doctest を実行する。
 *
 * 例:
 *   node cli.js -i README.n.md -o html=README.html
 *   node cli.js -i stdio.nepl -o test=out.json --jobs 2
 *
 * 出力:
 * - html: 静的 HTML
 * - test: JSON（summary + results）
 */

const fs = require("fs");
const path = require("path");
const { parseNmd } = require("./parser.js");
const { renderHtml } = require("./html_gen.js");
const { runAll } = require("./run_test.js");

function parseArgs(argv) {
    const inputs = [];
    const outputs = {}; // kind -> path
    let jobs = null;
    let compilerDir = "/mnt/data";

    let i = 2;
    while (i < argv.length) {
        const a = argv[i];
        if (a === "-i") {
            inputs.push(argv[i + 1]);
            i += 2;
            continue;
        }
        if (a === "-o") {
            const v = argv[i + 1] || "";
            const m = /^([a-zA-Z0-9_-]+)=(.+)$/.exec(v);
            if (!m) throw new Error(`-o expects kind=path, got: ${v}`);
            outputs[m[1]] = m[2];
            i += 2;
            continue;
        }
        if (a === "--no-hide-docpipe") {
            outputs["_hideDocPipe"] = "0";
            i += 1;
            continue;
        }
        if (a === "--title") {
            outputs["_title"] = argv[i + 1] || "";
            i += 2;
            continue;
        }
        if (a === "--jobs") {
            jobs = parseInt(argv[i + 1] || "1", 10);
            i += 2;
            continue;
        }
        if (a === "--compiler-dir") {
            compilerDir = argv[i + 1] || compilerDir;
            i += 2;
            continue;
        }
        throw new Error(`unknown arg: ${a}`);
    }
    return { inputs, outputs, jobs, compilerDir };
}

function ensureDirForFile(p) {
    const dir = path.dirname(p);
    fs.mkdirSync(dir, { recursive: true });
}

async function doHtml(inputs, outPath, opt) {
    if (!outPath) return;

    if (inputs.length === 1) {
        const src = fs.readFileSync(inputs[0], "utf-8");
        const ast = parseNmd(src);
        const html = renderHtml(ast, {
            hideDocPipe: opt.hideDocPipe,
            title: opt.title || path.basename(inputs[0]),
        });
        ensureDirForFile(outPath);
        fs.writeFileSync(outPath, html, "utf-8");
        return;
    }

    const outIsDir = fs.existsSync(outPath) ? fs.statSync(outPath).isDirectory() : !outPath.endsWith(".html");
    if (!outIsDir) throw new Error("multiple inputs require -o html=<directory>");
    fs.mkdirSync(outPath, { recursive: true });

    for (const inp of inputs) {
        if (!inp.endsWith(".n.md")) continue;
        const src = fs.readFileSync(inp, "utf-8");
        const ast = parseNmd(src);
        const html = renderHtml(ast, {
            hideDocPipe: opt.hideDocPipe,
            title: opt.title || path.basename(inp),
        });
        const outFile = path.join(outPath, path.basename(inp).replace(/\.n\.md$/, "") + ".html");
        fs.writeFileSync(outFile, html, "utf-8");
    }
}

async function doTest(inputs, outPath, compilerDir, jobs) {
    if (!outPath) return;
    const res = await runAll({ inputs, compilerDir, jobs });
    ensureDirForFile(outPath);
    fs.writeFileSync(outPath, JSON.stringify(res, null, 2), "utf-8");
    if (res.summary.failed > 0) process.exitCode = 1;
}

async function main() {
    const { inputs, outputs, jobs, compilerDir } = parseArgs(process.argv);
    if (inputs.length === 0) {
        console.error("no input: use -i file.n.md (or .nepl)");
        process.exit(2);
    }

    const hideDocPipe = outputs["_hideDocPipe"] !== "0";
    const title = outputs["_title"];

    // html は .n.md のみ
    await doHtml(inputs, outputs.html, { hideDocPipe, title });

    // test は .nepl と .n.md の両方
    await doTest(inputs, outputs.test, compilerDir, jobs);
}

main().catch((e) => {
    console.error(String(e && e.stack ? e.stack : e));
    process.exit(1);
});
