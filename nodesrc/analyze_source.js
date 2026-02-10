#!/usr/bin/env node
// nodesrc/analyze_source.js
// 目的:
// - nepl-web の analyze_lex / analyze_parse API を使って、lexer/parser の木構造と診断を確認する。
//
// 使い方:
//   node nodesrc/analyze_source.js --stage lex -i tests/functions.n.md
//   node nodesrc/analyze_source.js --stage parse -i path/to/file.nepl -o /tmp/parse.json

const fs = require('node:fs');
const path = require('node:path');
const { candidateDistDirs } = require('./util_paths');
const { loadCompilerFromCandidates } = require('./compiler_loader');
const { parseFile } = require('./parser');

function parseArgs(argv) {
    let inputPath = '';
    let outPath = '';
    let stage = 'parse';
    let distHint = '';
    let sourceInline = '';

    for (let i = 0; i < argv.length; i++) {
        const a = argv[i];
        if ((a === '-i' || a === '--input') && i + 1 < argv.length) {
            inputPath = argv[++i];
            continue;
        }
        if ((a === '-o' || a === '--output') && i + 1 < argv.length) {
            outPath = argv[++i];
            continue;
        }
        if (a === '--stage' && i + 1 < argv.length) {
            stage = argv[++i];
            continue;
        }
        if (a === '--dist' && i + 1 < argv.length) {
            distHint = argv[++i];
            continue;
        }
        if (a === '--source' && i + 1 < argv.length) {
            sourceInline = argv[++i];
            continue;
        }
        if (a === '-h' || a === '--help') {
            return { help: true, inputPath, outPath, stage, distHint, sourceInline };
        }
    }
    return { help: false, inputPath, outPath, stage, distHint, sourceInline };
}

function readSource(inputPath, sourceInline) {
    if (sourceInline) return sourceInline;
    if (!inputPath) throw new Error('input is required: -i <file> or --source <text>');
    const abs = path.resolve(inputPath);
    if (!fs.existsSync(abs)) throw new Error(`input not found: ${abs}`);

    if (abs.endsWith('.n.md')) {
        const parsed = parseFile(abs);
        const doctest = parsed.doctests?.[0];
        if (!doctest || !doctest.code) {
            throw new Error(`no doctest found in ${abs}`);
        }
        return doctest.code;
    }
    return fs.readFileSync(abs, 'utf-8');
}

function ensureDir(p) {
    fs.mkdirSync(p, { recursive: true });
}

function printSummary(result) {
    const tokens = Array.isArray(result?.tokens) ? result.tokens.length : 0;
    const diagnostics = Array.isArray(result?.diagnostics) ? result.diagnostics : [];
    const errors = diagnostics.filter(d => d?.severity === 'error').length;
    const warnings = diagnostics.filter(d => d?.severity === 'warning').length;
    const summary = {
        stage: result?.stage || null,
        ok: !!result?.ok,
        tokens,
        diagnostics: {
            total: diagnostics.length,
            errors,
            warnings,
        },
    };
    console.log(JSON.stringify(summary, null, 2));
}

async function main() {
    const { help, inputPath, outPath, stage, distHint, sourceInline } = parseArgs(process.argv.slice(2));
    if (help || !['lex', 'parse'].includes(stage)) {
        console.log('Usage: node nodesrc/analyze_source.js --stage <lex|parse> (-i <file> | --source <text>) [-o <out.json>] [--dist <dir>]');
        process.exit(help ? 0 : 2);
    }

    const source = readSource(inputPath, sourceInline);
    const candidates = candidateDistDirs(distHint || '');
    const loaded = await loadCompilerFromCandidates(candidates);
    const api = loaded.api;

    if (stage === 'lex' && typeof api.analyze_lex !== 'function') {
        throw new Error('compiler API analyze_lex is not available. rebuild dist first.');
    }
    if (stage === 'parse' && typeof api.analyze_parse !== 'function') {
        throw new Error('compiler API analyze_parse is not available. rebuild dist first.');
    }

    const result = stage === 'lex'
        ? api.analyze_lex(source)
        : api.analyze_parse(source);

    printSummary(result);
    if (outPath) {
        const outAbs = path.resolve(outPath);
        ensureDir(path.dirname(outAbs));
        fs.writeFileSync(outAbs, JSON.stringify(result, null, 2));
    }
}

if (require.main === module) {
    main().catch((e) => {
        console.error(String(e?.stack || e?.message || e));
        process.exit(1);
    });
}
