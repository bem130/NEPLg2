const { assert } = require('./_shared');

module.exports = {
    id: 'name_resolution_vfs_cross_file_definition_path',
    async run(api) {
        const source = `#entry main
#indent 4
#target core

#import "core/math" as *

fn main <()->i32> ():
    add 10 20
`;

        const result = api.analyze_name_resolution_with_vfs(
            '/virtual/main.nepl',
            source,
            {},
            { warn_important_shadow: true }
        );
        assert.equal(result?.stage, 'name_resolution', 'stage must be name_resolution');
        assert.equal(!!result?.ok, true, 'name resolution should be ok');

        const refs = Array.isArray(result?.references) ? result.references : [];
        assert.ok(refs.length > 0, 'references should exist');

        const addRef = refs.find(
            (r) =>
                r?.name === 'add' &&
                r?.resolved_def &&
                r.resolved_def?.span &&
                typeof r.resolved_def.span?.file_path === 'string'
        );
        assert.ok(addRef, 'add reference should include resolved_def span file_path');
        assert.ok(
            addRef.resolved_def.span.file_path.includes('/stdlib/core/math.nepl'),
            'resolved_def should point to stdlib/core/math.nepl'
        );

        return { checked: 5, reference_count: refs.length };
    },
};
