use nepl_core::diagnostic::Diagnostic;
use nepl_core::span::FileId;
use nepl_core::{compile_wasm, CompileOptions, CompileTarget};

mod harness;

fn compile_move_test(source: &str) -> Result<Vec<u8>, Vec<Diagnostic>> {
    let file_id = FileId(0);
    match compile_wasm(file_id, source, CompileOptions {
        target: Some(CompileTarget::Wasi),
    }) {
        Ok(artifact) => Ok(artifact.wasm),
        Err(nepl_core::error::CoreError::Diagnostics(ds)) => Err(ds),
        Err(_) => Err(Vec::new()),
    }
}

#[test]
fn move_simple_ok() {
    let source = r#"
#target wasi
#indent 4

fn main <()*>()>():
    let x <i32> 1;
    let y <i32> x; // x moved to y
    ()
"#;
    compile_move_test(source).expect("should succeed");
}

#[test]
fn move_use_after_move() {
    let source = r#"
#target wasi
#indent 4

fn main <()*>()>():
    let x <i32> 1;
    let y <i32> x; // x moved to y
    let z <i32> x; // error: use of moved value x
    ()
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs.iter().any(|d| d.message.contains("use of moved value")));
}

#[test]
fn move_in_branch() {
    let source = r#"
#target wasi
#indent 4

fn main <()*>()>():
    let x <i32> 1;
    if 1:
        let y <i32> x; // x moved
        ()
    else:
        ()
    let z <i32> x; // error: x moved in 'then' branch, so it's potentially moved
    ()
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs.iter().any(|d| d.message.contains("use of moved value")));
}

#[test]
fn move_in_loop() {
    // Loop moves variable from outer scope -> invalid after first iteration
    let source = r#"
#target wasi
#indent 4

fn main <()*>()>():
    let x <i32> 1;
    while 1:
        let y <i32> x; // moved in first iteration
        ()
    ()
"#;
    // This should fail because in the 2nd iteration (if it existed), x is moved.
    // Or if the loop runs once, checks pass.
    // But our analysis is: loop body runs, moves x.
    // Loop re-entry check: x is moved. ERROR "use of moved value" (in hypothetical 2nd iter).
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs.iter().any(|d| d.message.contains("use of moved value")));
}

#[test]
fn move_reassign() {
    let source = r#"
#target wasi
#indent 4

fn main <()*>()>():
    let x <i32> 1;
    let y <i32> x; // moved
    set x = 2;     // valid again
    let z <i32> x; // ok
    ()
"#;
    compile_move_test(source).expect("should succeed");
}
