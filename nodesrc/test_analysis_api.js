#!/usr/bin/env node
// nodesrc/test_analysis_api.js
// 目的:
// - nepl-web の解析 API（lex/parse/name_resolution/semantics）が期待どおりの形で
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

function getFnDef(parseResult, name) {
    const rootItems = parseResult?.module?.root?.items;
    if (!Array.isArray(rootItems)) return null;
    return rootItems.find((item) => item && item.kind === 'FnDef' && item.name === name) || null;
}

function getStmtExpr(stmt) {
    if (!stmt || typeof stmt !== 'object') return null;
    return stmt.expr && typeof stmt.expr === 'object' ? stmt.expr : null;
}

function firstSymbolDebug(expr) {
    if (!expr || !Array.isArray(expr.items) || expr.items.length === 0) return '';
    const first = expr.items[0];
    return String(first?.kind === 'Symbol' ? (first.debug || '') : '');
}

function assertItemKinds(expr, expectedKinds, label) {
    const items = Array.isArray(expr?.items) ? expr.items : [];
    const actualKinds = items.map((x) => x.kind);
    if (actualKinds.length !== expectedKinds.length) {
        fail(`${label}: expected item count ${expectedKinds.length}, got ${actualKinds.length} (${actualKinds.join(',')})`);
    }
    for (let i = 0; i < expectedKinds.length; i++) {
        if (actualKinds[i] !== expectedKinds[i]) {
            fail(`${label}: expected item[${i}]=${expectedKinds[i]}, got ${actualKinds[i]}`);
        }
    }
}

async function run() {
    const loaded = await loadCompilerFromCandidates(candidateDistDirs(''));
    const api = loaded.api;

    if (typeof api.analyze_lex !== 'function') fail('analyze_lex is missing');
    if (typeof api.analyze_parse !== 'function') fail('analyze_parse is missing');
    if (typeof api.analyze_name_resolution !== 'function') fail('analyze_name_resolution is missing');
    if (typeof api.analyze_semantics !== 'function') fail('analyze_semantics is missing');

    const cases = [
        {
            id: 'semantics_token_type_and_ranges',
            source: `#entry main
#indent 4
#target wasm
fn main <()->i32> ():
    1
`,
            checkSemantics(semResult) {
                const exprs = Array.isArray(semResult?.expressions) ? semResult.expressions : [];
                if (exprs.length === 0) fail('semantics_token_type_and_ranges: expressions missing');
                const lit = exprs.find(e => e && e.kind === 'LiteralI32');
                if (!lit) fail('semantics_token_type_and_ranges: literal expression missing');
                if (!Array.isArray(lit.argument_ranges) || lit.argument_ranges.length !== 0) {
                    fail('semantics_token_type_and_ranges: literal argument ranges should be empty');
                }
                const tokenSem = Array.isArray(semResult?.token_semantics) ? semResult.token_semantics : [];
                if (tokenSem.length === 0) fail('semantics_token_type_and_ranges: token_semantics missing');
                const hasTypedToken = tokenSem.some(t => t && typeof t.inferred_type === 'string' && t.inferred_type.length > 0);
                if (!hasTypedToken) fail('semantics_token_type_and_ranges: inferred_type not found on any token');
            },
        },
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
        {
            id: 'parse_if_inline_no_colon_blocks',
            source: `#entry main
#indent 4
#target wasm
fn main <()->i32> ():
    if cond true then 1 else 2
`,
            checkParse(parseResult) {
                const fn = getFnDef(parseResult, 'main');
                if (!fn) fail('parse_if_inline_no_colon_blocks: main not found');
                const bodyItems = Array.isArray(fn?.body?.items) ? fn.body.items : [];
                const ifStmt = bodyItems.find((stmt) => firstSymbolDebug(getStmtExpr(stmt)).startsWith('If('));
                if (!ifStmt) fail('parse_if_inline_no_colon_blocks: if stmt not found');
                const ifExpr = getStmtExpr(ifStmt);
                assertItemKinds(ifExpr, ['Symbol', 'Literal', 'Literal', 'Literal'], 'parse_if_inline_no_colon_blocks');
            },
        },
        {
            id: 'parse_if_colon_uses_block_for_cond_then_else',
            source: `#entry main
#indent 4
#target wasm
fn main <()->i32> ():
    if:
        cond:
            true
        then:
            1
        else:
            2
`,
            checkParse(parseResult) {
                const fn = getFnDef(parseResult, 'main');
                if (!fn) fail('parse_if_colon_uses_block_for_cond_then_else: main not found');
                const bodyItems = Array.isArray(fn?.body?.items) ? fn.body.items : [];
                const ifStmt = bodyItems.find((stmt) => firstSymbolDebug(getStmtExpr(stmt)).startsWith('If('));
                if (!ifStmt) fail('parse_if_colon_uses_block_for_cond_then_else: if stmt not found');
                const ifExpr = getStmtExpr(ifStmt);
                assertItemKinds(ifExpr, ['Symbol', 'Block', 'Block', 'Block'], 'parse_if_colon_uses_block_for_cond_then_else');
            },
        },
        {
            id: 'parse_while_inline_no_colon_blocks',
            source: `#entry main
#indent 4
#target wasm
fn main <()->i32> ():
    let mut i 0;
    while cond lt i 3 do set i add i 1;
    i
`,
            checkParse(parseResult) {
                const fn = getFnDef(parseResult, 'main');
                if (!fn) fail('parse_while_inline_no_colon_blocks: main not found');
                const bodyItems = Array.isArray(fn?.body?.items) ? fn.body.items : [];
                const whileStmt = bodyItems.find((stmt) => firstSymbolDebug(getStmtExpr(stmt)).startsWith('While('));
                if (!whileStmt) fail('parse_while_inline_no_colon_blocks: while stmt not found');
                const whileExpr = getStmtExpr(whileStmt);
                const kinds = Array.isArray(whileExpr?.items) ? whileExpr.items.map((x) => x.kind) : [];
                if (kinds.includes('Block')) {
                    fail(`parse_while_inline_no_colon_blocks: unexpected Block in inline while (${kinds.join(',')})`);
                }
            },
        },
        {
            id: 'parse_while_colon_uses_block_for_cond_do',
            source: `#entry main
#indent 4
#target wasm
fn main <()->i32> ():
    let mut i 0;
    while:
        cond lt i 3
        do:
            set i add i 1;
    i
`,
            checkParse(parseResult) {
                const fn = getFnDef(parseResult, 'main');
                if (!fn) fail('parse_while_colon_uses_block_for_cond_do: main not found');
                const bodyItems = Array.isArray(fn?.body?.items) ? fn.body.items : [];
                const whileStmt = bodyItems.find((stmt) => firstSymbolDebug(getStmtExpr(stmt)).startsWith('While('));
                if (!whileStmt) fail('parse_while_colon_uses_block_for_cond_do: while stmt not found');
                const whileExpr = getStmtExpr(whileStmt);
                assertItemKinds(whileExpr, ['Symbol', 'Block', 'Block'], 'parse_while_colon_uses_block_for_cond_do');
            },
        },
    ];

    const results = [];
    for (const c of cases) {
        const lex = api.analyze_lex(c.source);
        const parse = api.analyze_parse(c.source);
        const resolve = api.analyze_name_resolution(c.source);
        const semantics = api.analyze_semantics(c.source);
        if (!lex || lex.stage !== 'lex') fail(`${c.id}: lex stage mismatch`);
        if (!parse || parse.stage !== 'parse') fail(`${c.id}: parse stage mismatch`);
        if (!resolve || resolve.stage !== 'name_resolution') fail(`${c.id}: resolve stage mismatch`);
        if (!semantics || semantics.stage !== 'semantics') fail(`${c.id}: semantics stage mismatch`);
        if (!resolve.ok) fail(`${c.id}: resolve not ok`);
        if (!Array.isArray(semantics?.diagnostics)) fail(`${c.id}: semantics diagnostics missing`);
        if (typeof c.checkParse === 'function') c.checkParse(parse);
        if (typeof c.check === 'function') c.check(resolve);
        if (typeof c.checkSemantics === 'function') c.checkSemantics(semantics);
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
