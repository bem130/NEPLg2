# stdlib/string.n.md

## string_main

neplg2:test
```neplg2

#entry main
#indent 4
#target std

#import "alloc/string" as *
#import "alloc/vec" as *
#import "core/option" as *
#import "std/test" as *

fn main <()*> ()> ():
    // Test len with literals
    assert_eq_i32 3 len "abc";
    assert_eq_i32 0 len "";
    assert_eq_i32 5 len "hello";

    // Test str_eq with same strings
    assert_str_eq "hello" "hello";
    assert_str_eq "" "";
    assert_str_eq "a" "a";

    // Test str_eq with different strings
    assert_ne true str_eq "hello" "world";
    assert_ne true str_eq "a" "b";
    assert_ne true str_eq "abc" "ab";

    // Test from_i32
    let s0 from_i32 0;
    assert_eq_i32 1 len s0;

    let s1234 from_i32 1234;
    assert_eq_i32 4 len s1234;

    let s42 from_i32 42;
    assert_eq_i32 2 len s42;

    // Test concat (basic strings)
    let greeting:
        "hello"
        |> concat "world";
    assert_eq_i32 10 len greeting;

    // Test concat with empty strings
    let empty "";
    let test:
        empty
        |> concat "test";
    assert_eq_i32 4 len test;

    // Test trim
    let src "  fn main(a: i32)  ";
    let trimmed str_trim src;
    assert_str_eq "fn main(a: i32)" trimmed;

    // Test starts_with/ends_with
    assert str_starts_with trimmed "fn";
    assert str_ends_with trimmed ")";
    assert_ne true str_starts_with trimmed "let";

    // Test slice
    let fname str_slice trimmed 3 7;
    assert_str_eq "main" fname;

    // Test split (single char)
    let parts str_split trimmed "(";
    assert_eq_i32 2 vec_len<str> parts;
    let parts0 str_split trimmed "(";
    let p0 unwrap<str> vec_get<str> parts0 0;
    let parts1 str_split trimmed "(";
    let p1 unwrap<str> vec_get<str> parts1 1;
    assert_str_eq "fn main" p0;
    assert_str_eq "a: i32)" p1;

    // Test split (multi char)
    let parts2 str_split "a--b--c" "--";
    assert_eq_i32 3 vec_len<str> parts2;
    let parts2_0 str_split "a--b--c" "--";
    let p2_0 unwrap<str> vec_get<str> parts2_0 0;
    let parts2_2 str_split "a--b--c" "--";
    let p2_2 unwrap<str> vec_get<str> parts2_2 2;
    assert_str_eq "a" p2_0;
    assert_str_eq "c" p2_2;

    // Test split (empty delimiter)
    let parts3 str_split "abc" "";
    assert_eq_i32 1 vec_len<str> parts3;

    // Test StringBuilder
    let msg <str>:
        string_builder_new
        |> sb_append "Error: "
        |> sb_append_i32 404
        |> sb_append " Not Found"
        |> sb_build;
    assert_str_eq "Error: 404 Not Found" msg;
    assert_eq_i32 20 len msg;

    let lines <str>:
        string_builder_new
        |> sb_append_line "first"
        |> sb_append_line "second"
        |> sb_build;
    assert_str_eq "first\nsecond\n" lines;

    ()
```
