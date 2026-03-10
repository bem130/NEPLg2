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
