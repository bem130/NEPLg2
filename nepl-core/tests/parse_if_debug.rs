use nepl_core::lexer;
use nepl_core::parser;
use nepl_core::span::FileId;
use std::path::PathBuf;

#[test]
fn parse_if_debug() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "std/math"
#use std::math::*

fn main <()->i32> ():
    let v <i32> if:
        cond lt 2 3
        7
        8
    v
"#;

    let file_id = FileId(0);
    let lex = lexer::lex(file_id, src);
    println!("Lexer diagnostics: {:#?}", lex.diagnostics);
    let parse = parser::parse_tokens(file_id, lex);
    println!("Parse diagnostics: {:#?}", parse.diagnostics);
    println!("Parse module: {:#?}", parse.module);
}
