const { assert } = require('./_shared');

function compileWithProfile(api, source, profile) {
    if (typeof api.compile_source_with_profile !== 'function') {
        throw new Error('compile_source_with_profile API is not available');
    }
    return api.compile_source_with_profile(source, profile);
}

function expectCompileFail(fn, needle) {
    let threw = false;
    try {
        fn();
    } catch (e) {
        threw = true;
        const msg = String(e?.message || e || '');
        if (needle) {
            assert.ok(msg.includes(needle), `error must include '${needle}', got: ${msg}`);
        }
    }
    assert.equal(threw, true, 'expected compilation to fail');
}

module.exports = {
    id: 'profile_gate_compile_debug_release',
    async run(api) {
        const debugOnlySource = `#entry main
#if[profile=debug]
fn only_debug <()->i32> ():
    123

fn main <()->i32> ():
    only_debug
`;

        const releaseOnlySource = `#entry main
#if[profile=release]
fn only_release <()->i32> ():
    456

fn main <()->i32> ():
    only_release
`;

        const releaseSkipUnknownSource = `#entry main
#if[profile=release]
fn only_release_bad <()->i32> ():
    unknown_symbol

fn main <()->i32> ():
    0
`;

        const wasmDebug = compileWithProfile(api, debugOnlySource, 'debug');
        assert.ok(wasmDebug instanceof Uint8Array, 'debug profile must compile debug-gated definition');

        expectCompileFail(
            () => compileWithProfile(api, debugOnlySource, 'release'),
            'undefined identifier'
        );

        const wasmRelease = compileWithProfile(api, releaseOnlySource, 'release');
        assert.ok(wasmRelease instanceof Uint8Array, 'release profile must compile release-gated definition');

        expectCompileFail(
            () => compileWithProfile(api, releaseOnlySource, 'debug'),
            'undefined identifier'
        );

        const wasmReleaseSkip = compileWithProfile(api, releaseSkipUnknownSource, 'debug');
        assert.ok(
            wasmReleaseSkip instanceof Uint8Array,
            'debug profile must skip release-only invalid definition'
        );

        return { checked: 5 };
    },
};
