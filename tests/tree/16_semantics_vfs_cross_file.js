const { assert } = require('./_shared');

module.exports = {
    id: 'semantics_vfs_cross_file_definition_path',
    async run(api) {
        const source = `#entry main
#indent 4
#target core

#import "core/math" as *

fn main <()->i32> ():
    add 1 2
`;

        const result = api.analyze_semantics_with_vfs('/virtual/main.nepl', source, {});
        assert.equal(result?.stage, 'semantics', 'stage must be semantics');
        assert.equal(!!result?.ok, true, 'semantics should be ok');

        const tokenRes = Array.isArray(result?.token_resolution) ? result.token_resolution : [];
        assert.ok(tokenRes.length > 0, 'token_resolution should exist');

        const addRef = tokenRes.find(
            (t) =>
                t?.name === 'add' &&
                t?.resolved_definition &&
                t.resolved_definition?.span &&
                typeof t.resolved_definition.span?.file_path === 'string'
        );
        assert.ok(addRef, 'add reference should resolve to definition with file_path');
        assert.ok(
            addRef.resolved_definition.span.file_path.includes('/stdlib/core/math.nepl'),
            'resolved definition should point to stdlib/core/math.nepl'
        );

        const candidates = Array.isArray(addRef?.candidate_definitions)
            ? addRef.candidate_definitions
            : [];
        assert.ok(candidates.length > 0, 'candidate_definitions should exist');
        assert.ok(
            candidates.some(
                (c) =>
                    c &&
                    c.span &&
                    typeof c.span.file_path === 'string' &&
                    c.span.file_path.includes('/stdlib/core/math.nepl')
            ),
            'candidate_definitions should include stdlib/core/math.nepl entry'
        );

        return { checked: 7, token_resolution_count: tokenRes.length };
    },
};
