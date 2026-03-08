const { assert } = require('./_shared');

module.exports = {
    id: 'wasm_unreachable_function_pruning',
    async run(api) {
        const source = `#entry main
#indent 4
#target core
#import "core/math" as *

fn dead <()->i32> ():
    add 1 2

fn live <()->i32> ():
    add 3 4

fn main <()->i32> ():
    live
`;

        const out = api.compile_outputs(source, ['wat'], false);
        const wat = String(out?.wat || '');
        assert.ok(wat.length > 0, 'wat should be generated');
        assert.equal(wat.includes('live__unit__i32__pure'), true, 'reachable function should be emitted');
        assert.equal(wat.includes('dead__unit__i32__pure'), false, 'unreachable function should be pruned');

        return {
            checked: 3,
            wat_length: wat.length,
            has_live: wat.includes('live__unit__i32__pure'),
            has_dead: wat.includes('dead__unit__i32__pure'),
        };
    },
};

