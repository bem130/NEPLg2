const { assert } = require('./_shared');

module.exports = {
    id: 'diagnostic_ids_for_target_and_loader',
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

        return { checked: 5, diagnostics_count: diagnostics.length + missingDs.length };
    },
};
