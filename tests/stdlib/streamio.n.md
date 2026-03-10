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
#import "alloc/collections/vec" as *
#import "core/result" as *

fn main <()*>i32> ():
    let mut bytes0 <Vec<u8>> vec_new<u8>
    set bytes0 vec_push<u8> bytes0 <u8> cast 65
    set bytes0 vec_push<u8> bytes0 <u8> cast 66
    set bytes0 vec_push<u8> bytes0 <u8> cast 10
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
