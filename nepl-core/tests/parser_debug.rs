use std::fs;
use std::path::PathBuf;

#[test]
fn debug_parse_string_nepl() {
    use nepl_core::{lexer, parser, span::FileId};

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("stdlib")
        .join("alloc")
        .join("string.nepl");
    let src = fs::read_to_string(&path).expect("failed to read alloc/string.nepl");
    let lex = lexer::lex(FileId(3), &src);
    if !lex.diagnostics.is_empty() {
        println!("LEX DIAGS: {:?}", lex.diagnostics);
    }
    let parse = parser::parse_tokens(FileId(3), lex);
    println!("PARSE DIAGS: {:?}", parse.diagnostics);
    println!("MODULE AST: {:#?}", parse.module);
    assert!(parse.module.is_some());
}
