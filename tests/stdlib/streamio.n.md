# streamio facade

## stdout_text_writer_via_trait_helper

neplg2:test
stdout: "stream text\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
#import "core/result" as *

fn main <()*>i32> ():
    match stream_write_str stdout_stream "stream text\n":
        Result::Ok out:
            match stream_flush out:
                Result::Ok _:
                    0
                Result::Err _e:
                    1
        Result::Err _e:
            1
```

## stdout_binary_writer_roundtrip

neplg2:test
stdout: "AB\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
#import "core/result" as *

fn main <()*>i32> ():
    let bytes0 <ByteBuf> stream_bytes_from_str "AB\n"
    match stream_write_bytes stdout_stream bytes0:
        Result::Ok out:
            match stream_flush out:
                Result::Ok _:
                    0
                Result::Err _e:
                    1
        Result::Err _e:
            1
```

## stdout_binary_writer_preserves_nul

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
#import "std/test" as *

fn main <()*>i32> ():
    let bytes0 <ByteBuf> stream_bytes_from_str "A\x00B\n"
    let text <str> stream_bytes_to_str bytes0
    assert_str_eq "A\x00B\n" text
    0
```

## stream_writer_text_and_i32

neplg2:test
stdout: "sum=42\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
#import "core/result" as *

fn main <()*>i32> ():
    let mut w <StreamWriter> unwrap_ok stream_writer_new;
    set w stream_writer_write_str w "sum=";
    set w stream_writer_write_i32_ln w 42;
    set w stream_writer_flush w;
    stream_writer_free w;
    0
```

## stream_writer_space_and_i64

neplg2:test
stdout: "1 2\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
#import "core/cast" as *

fn main <()*>i32> ():
    let mut w <StreamWriter> unwrap_ok stream_writer_new;
    set w stream_writer_write_i32 w 1;
    set w stream_writer_write_space w;
    set w stream_writer_write_i64_ln w <i64> cast 2;
    set w stream_writer_flush w;
    stream_writer_free w;
    0
```

## stdin_binary_reader_to_stdout

neplg2:test
stdin: "line1\nline2"
stdout: "line1\nline2"
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
#import "core/result" as *

fn main <()*>i32> ():
    match stream_read_all_bytes stdin_stream:
        Result::Ok bytes:
            match stream_write_bytes stdout_stream bytes:
                Result::Ok out:
                    match stream_flush out:
                        Result::Ok _:
                            0
                        Result::Err _e:
                            1
                Result::Err _e:
                    1
        Result::Err _e:
            1
```

## stream_scanner_reads_numbers

neplg2:test[normalize_newlines]
stdin: "10 -20 +30 4.5\n"
stdout: "10\n-20\n30\n4.500000\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
fn main <()*>i32> ():
    let sc <StreamScanner> unwrap_ok stream_scanner_new;
    let a <i32> stream_scanner_read_i32 sc;
    let b <i32> stream_scanner_read_i32 sc;
    let c <i64> stream_scanner_read_u64 sc;
    let d <f64> stream_scanner_read_f64 sc;
    let mut w <StreamWriter> unwrap_ok stream_writer_new;
    set w stream_writer_write_i32_ln w a;
    set w stream_writer_write_i32_ln w b;
    set w stream_writer_write_i64_ln w c;
    set w stream_writer_write_f64_ln w d;
    set w stream_writer_flush w;
    stream_writer_free w;
    0
```

## stream_scanner_skips_bom_and_token

neplg2:test[normalize_newlines]
stdin: "\ufeffabc 42\n"
stdout: "abc\n42\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
#import "std/stdio" as *

fn main <()*>i32> ():
    let sc <StreamScanner> unwrap_ok stream_scanner_new;
    let token <str> stream_scanner_read_token sc;
    let value <i32> stream_scanner_read_i32 sc;
    print token;
    println "";
    println_i32 value;
    0
```
