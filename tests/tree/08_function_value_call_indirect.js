const { assert } = require('./_shared');

module.exports = {
    id: 'function_value_call_indirect',
    async run(api) {
        const source = `#entry main
#indent 4
#target wasm

fn inc <(i32)->i32> (x):
    x

fn apply <((i32)->i32, i32)->i32> (f, x):
    f x

fn main <()->i32> ():
    let f @inc;
    apply f 41
`;

        const parse = api.analyze_parse(source);
        assert.equal(!!parse?.ok, true, 'parse should succeed');
        const tokens = Array.isArray(parse?.tokens) ? parse.tokens : [];
        const hasAt = tokens.some((t) => t?.kind === 'At');
        assert.ok(
            hasAt,
            '@fn syntax should produce At token'
        );

        const sem = api.analyze_semantics(source);
        assert.equal(!!sem?.ok, true, 'semantics should succeed');
        const exprs = Array.isArray(sem?.expressions) ? sem.expressions : [];
        const hasIndirect = exprs.some((e) => e?.kind === 'CallIndirect');
        assert.ok(hasIndirect, 'function value call should lower to CallIndirect');

        return {
            checked: 3,
            expr_count: exprs.length,
            has_call_indirect: hasIndirect,
        };
    },
};
