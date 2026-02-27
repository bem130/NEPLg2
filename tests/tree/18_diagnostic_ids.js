const { assert } = require('./_shared');

module.exports = {
    id: 'diagnostic_ids_for_target_loader_parser_typecheck',
    async run(api) {
        const badTarget = `#entry main
#indent 4
#target wasi2

fn main <()->i32> ():
    0
`;
        const sem = api.analyze_semantics(badTarget);
        const diagnostics = Array.isArray(sem?.diagnostics) ? sem.diagnostics : [];
        const unknownTarget = diagnostics.find(
            (d) =>
                d?.severity === 'error' &&
                d?.message === 'unknown target in #target' &&
                d?.id === 1002
        );
        assert.ok(unknownTarget, 'unknown target diagnostic should include id=1002');
        assert.equal(
            unknownTarget?.id_message,
            'unknown target in #target',
            'id_message should resolve from diagnostic table'
        );

        const missing = api.analyze_name_resolution_with_vfs(
            '/virtual/missing.nepl',
            '#entry main\n#indent 4\n#target core\n#import \"missing/module\" as *\nfn main <()->i32> (): 0\n',
            {},
            { warn_important_shadow: true }
        );
        assert.equal(!!missing?.ok, false, 'missing module should fail');
        const missingDs = Array.isArray(missing?.diagnostics) ? missing.diagnostics : [];
        const loaderDiag = missingDs.find((d) => d?.id === 1003);
        assert.ok(loaderDiag, 'loader diagnostic should include id=1003');

        const parseBad = `#entry main
#indent 4
#target core

fn main <()->i32> ():
    let
`;
        const parseRes = api.analyze_semantics(parseBad);
        const parseDs = Array.isArray(parseRes?.diagnostics) ? parseRes.diagnostics : [];
        assert.ok(
            parseDs.some((d) => d?.id === 2003),
            'parse diagnostics should include parser expected-identifier id=2003'
        );

        const undefVar = `#entry main
#indent 4
#target core
fn main <()->i32> ():
    unknown_symbol
`;
        const undefRes = api.analyze_semantics(undefVar);
        const undefDs = Array.isArray(undefRes?.diagnostics) ? undefRes.diagnostics : [];
        assert.ok(
            undefDs.some((d) => d?.id === 3001),
            'typecheck diagnostics should include undefined identifier id=3001'
        );

        const overloadAmb = `#entry main
#indent 4
#target core

fn cast <(i32)->i32> (x): x
fn cast <(i32)->f32> (x): i32_to_f32 x
fn main <()->i32> ():
    let y cast 1
    0
`;
        const overloadRes = api.analyze_semantics(overloadAmb);
        const overloadDs = Array.isArray(overloadRes?.diagnostics) ? overloadRes.diagnostics : [];
        assert.ok(
            overloadDs.some((d) => d?.id === 3005),
            'overload diagnostics should include ambiguous overload id=3005'
        );

        return {
            checked: 8,
            diagnostics_count:
                diagnostics.length + missingDs.length + parseDs.length + undefDs.length + overloadDs.length,
        };
    },
};
