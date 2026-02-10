#!/usr/bin/env node
// nodesrc/test_analysis_api.js
// 目的:
// - nepl-web の解析 API（lex/parse/name_resolution）が期待どおりの形で
//   情報を返せることを確認する。

const { candidateDistDirs } = require('./util_paths');
const { loadCompilerFromCandidates } = require('./compiler_loader');

function fail(msg) {
    throw new Error(msg);
}

function findRefByName(result, name) {
    const refs = Array.isArray(result?.references) ? result.references : [];
    return refs.filter(r => r && r.name === name);
}

async function run() {
    const loaded = await loadCompilerFromCandidates(candidateDistDirs(''));
    const api = loaded.api;

    if (typeof api.analyze_lex !== 'function') fail('analyze_lex is missing');
    if (typeof api.analyze_parse !== 'function') fail('analyze_parse is missing');
    if (typeof api.analyze_name_resolution !== 'function') fail('analyze_name_resolution is missing');

    const cases = [
        {
            id: 'shadowing_local_let',
            source: `#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let x 1;
    let x 2;
    add x 3
`,
            check(resolveResult) {
                const defs = Array.isArray(resolveResult.definitions) ? resolveResult.definitions : [];
                const xDefs = defs.filter(d => d.name === 'x');
                if (xDefs.length < 2) fail('expected at least two definitions for x');
                const refs = findRefByName(resolveResult, 'x');
                if (refs.length === 0) fail('expected a reference to x');
                const lastRef = refs[refs.length - 1];
                const newestDefId = xDefs[xDefs.length - 1].id;
                if (lastRef.resolved_def_id !== newestDefId) {
                    fail(`expected x to resolve to newest def ${newestDefId}, got ${lastRef.resolved_def_id}`);
                }
            },
        },
        {
            id: 'fn_alias_target_resolution',
            source: `#entry main
#indent 4
#target wasm
#import "core/math" as *

fn inc <(i32)->i32> (x):
    add x 1

fn plus @inc;

fn main <()->i32> ():
    plus 41
`,
            check(resolveResult) {
                const refs = findRefByName(resolveResult, 'inc');
                if (refs.length === 0) fail('expected alias target reference to inc');
                if (refs[0].resolved_def_id == null) fail('alias target inc should be resolved');
            },
        },
    ];

    const results = [];
    for (const c of cases) {
        const lex = api.analyze_lex(c.source);
        const parse = api.analyze_parse(c.source);
        const resolve = api.analyze_name_resolution(c.source);
        if (!lex || lex.stage !== 'lex') fail(`${c.id}: lex stage mismatch`);
        if (!parse || parse.stage !== 'parse') fail(`${c.id}: parse stage mismatch`);
        if (!resolve || resolve.stage !== 'name_resolution') fail(`${c.id}: resolve stage mismatch`);
        if (!resolve.ok) fail(`${c.id}: resolve not ok`);
        c.check(resolve);
        results.push({ id: c.id, ok: true });
    }

    const out = {
        summary: {
            total: results.length,
            passed: results.length,
            failed: 0,
        },
        results,
    };
    console.log(JSON.stringify(out, null, 2));
}

run().catch((e) => {
    console.error(String(e?.stack || e?.message || e));
    process.exit(1);
});
