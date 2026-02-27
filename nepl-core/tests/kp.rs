mod harness;

use harness::run_main_capture_stdout_with_stdin;

#[test]
fn kpread_to_stdio_stdout_i32() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "kp/kpread" as *
#import "std/stdio" as *

fn main <()*>()> ():
    let sc <i32> scanner_new;
    println_i32 scanner_read_i32 sc;
    println_i32 scanner_read_i32 sc;
    println_i32 scanner_read_i32 sc;
"#;
    let out = run_main_capture_stdout_with_stdin(src, b"10 20 30\n");
    assert_eq!(out, "10\n20\n30\n");
}

#[test]
fn stdio_stdin_to_kpwrite_stdout() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "std/stdio" as *
#import "kp/kpwrite" as *

fn main <()*>()> ():
    let line <str> read_line;
    let w <i32> writer_new;
    writer_write_str w line;
    writer_flush w;
    writer_free w;
"#;
    let out = run_main_capture_stdout_with_stdin(src, b"hello world\n");
    assert_eq!(out, "hello world");
}

#[test]
fn kpread_to_kpwrite_prefixsum_i32() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "core/math" as *
#import "core/mem" as *
#import "kp/kpread" as *
#import "kp/kpwrite" as *

fn main <()*>()> ():
    let sc <i32> scanner_new;
    let n <i32> scanner_read_i32 sc;
    let q <i32> scanner_read_i32 sc;

    let pref_len <i32> add n 1;
    let pref <i32> alloc mul pref_len 4;
    store_i32 pref 0;

    let mut i <i32> 1;
    while le i n:
        do:
            let a <i32> scanner_read_i32 sc;
            let prev_off <i32> mul sub i 1 4;
            let prev_ptr <i32> add pref prev_off;
            let prev <i32> load_i32 prev_ptr;
            let cur <i32> add prev a;
            let cur_off <i32> mul i 4;
            let cur_ptr <i32> add pref cur_off;
            store_i32 cur_ptr cur;
            set i add i 1;

    let w <i32> writer_new;
    let mut k <i32> 0;
    while lt k q:
        do:
            let l1 <i32> scanner_read_i32 sc;
            let r1 <i32> scanner_read_i32 sc;
            let l <i32> sub l1 1;
            let left_off <i32> mul l 4;
            let right_off <i32> mul r1 4;
            let left <i32> load_i32 add pref left_off;
            let right <i32> load_i32 add pref right_off;
            writer_write_i32 w sub right left;
            writer_writeln w;
            set k add k 1;

    writer_flush w;
    writer_free w;
    dealloc pref mul pref_len 4;
"#;
    let out =
        run_main_capture_stdout_with_stdin(src, b"5 3\n1 2 3 4 5\n1 3\n2 5\n1 5\n");
    assert_eq!(out, "6\n14\n15\n");
}

#[test]
fn kpread_to_kpwrite_i64_dp() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "core/math" as *
#import "kp/kpread" as *
#import "kp/kpwrite" as *

fn ways <(i32)*>i64> (n):
    if le n 1:
        then i64_extend_i32_u 1
        else:
            let mut a <i64> i64_extend_i32_u 1;
            let mut b <i64> i64_extend_i32_u 1;
            let mut i <i32> 2;
            while le i n:
                do:
                    let c <i64> i64_add a b;
                    set a b;
                    set b c;
                    set i add i 1;
            b

fn main <()*>()> ():
    let sc <i32> scanner_new;
    let n <i32> scanner_read_i32 sc;
    let ans <i64> ways n;
    let w <i32> writer_new;
    writer_write_i64 w ans;
    writer_writeln w;
    writer_flush w;
    writer_free w;
"#;
    let out = run_main_capture_stdout_with_stdin(src, b"6\n");
    assert_eq!(out, "13\n");
}

#[test]
fn kpwrite_i64_stdout_no_input() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "core/math" as *
#import "kp/kpwrite" as *

fn main <()*>()> ():
    let w <i32> writer_new;
    writer_write_i64 w i64_extend_i32_u 13;
    writer_writeln w;
    writer_flush w;
    writer_free w;
"#;
    let out = run_main_capture_stdout_with_stdin(src, b"");
    assert_eq!(out, "13\n");
}

#[test]
fn kpwrite_i32_lines_no_input() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "kp/kpwrite" as *

fn main <()*>()> ():
    let w <i32> writer_new;
    writer_write_i32 w 6;
    writer_writeln w;
    writer_write_i32 w 14;
    writer_writeln w;
    writer_write_i32 w 15;
    writer_writeln w;
    writer_flush w;
    writer_free w;
"#;
    let out = run_main_capture_stdout_with_stdin(src, b"");
    assert_eq!(out, "6\n14\n15\n");
}

#[test]
fn kpread_to_kpwrite_f64() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "kp/kpread" as *
#import "kp/kpwrite" as *

fn main <()*>()> ():
    let sc <i32> scanner_new;
    let a <f64> scanner_read_f64 sc;
    let b <f64> scanner_read_f64 sc;
    let c <f64> scanner_read_f64 sc;
    let w <i32> writer_new;
    writer_write_f64_ln w a;
    writer_write_f64_ln w b;
    writer_write_f64_ln w c;
    writer_flush w;
    writer_free w;
"#;
    let out = run_main_capture_stdout_with_stdin(src, b"3.5 -2.25 1e2\n");
    assert_eq!(out, "3.500000\n-2.250000\n100.000000\n");
}

#[test]
fn kpread_to_kpwrite_f32() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "kp/kpread" as *
#import "kp/kpwrite" as *

fn main <()*>()> ():
    let sc <i32> scanner_new;
    let v <f32> scanner_read_f32 sc;
    let w <i32> writer_new;
    writer_write_f32_ln w v;
    writer_flush w;
    writer_free w;
"#;
    let out = run_main_capture_stdout_with_stdin(src, b"1.25\n");
    assert_eq!(out, "1.250000\n");
}

#[test]
fn kpread_scanner_header_debug() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "core/mem" as *
#import "std/stdio" as *
#import "kp/kpread" as *

fn main <()*>()> ():
    let sc <i32> scanner_new;
    let buf <i32> load_i32 sc;
    let len <i32> load_i32 add sc 4;
    let pos <i32> load_i32 add sc 8;
    print_i32 sc;
    print " ";
    print_i32 buf;
    print " ";
    print_i32 len;
    print " ";
    println_i32 pos;
"#;
    let out = run_main_capture_stdout_with_stdin(src, b"10 20 30\n");
    println!("kpread_scanner_header_debug out={out:?}");
    let parts: Vec<&str> = out.trim().split(' ').collect();
    assert_eq!(parts.len(), 4, "unexpected scanner header format: {out}");
    let sc: i32 = parts[0].parse().expect("sc parse");
    let buf: i32 = parts[1].parse().expect("buf parse");
    let len: i32 = parts[2].parse().expect("len parse");
    let pos: i32 = parts[3].parse().expect("pos parse");
    assert!(sc > 0, "scanner ptr should be non-zero: {out}");
    assert!(buf > 0, "buffer ptr should be non-zero: {out}");
    assert_ne!(sc, buf, "scanner header ptr and buffer ptr must differ: {out}");
    assert!(len > 0, "buffer len should be >0: {out}");
    assert_eq!(pos, 0, "initial position should be 0: {out}");
}

#[test]
fn kpread_buffer_bytes_debug() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "core/mem" as *
#import "std/stdio" as *
#import "kp/kpread" as *

fn main <()*>()> ():
    let sc <i32> scanner_new;
    let buf <i32> load_i32 sc;
    let len <i32> load_i32 add sc 4;
    let p0 <i32> buf;
    let p1 <i32> add buf 1;
    let p2 <i32> add buf 2;
    let b0 <i32> if lt 0 len load_u8 p0 -1;
    let b1 <i32> if lt 1 len load_u8 p1 -1;
    let b2 <i32> if lt 2 len load_u8 p2 -1;
    print_i32 sc;
    print " ";
    print_i32 buf;
    print " ";
    print_i32 len;
    print " ";
    print_i32 b0;
    print " ";
    print_i32 b1;
    print " ";
    print_i32 b2;
    println "";
"#;
    let out = run_main_capture_stdout_with_stdin(src, b"10 20 30\n");
    println!("local_scanner_new_logic_debug out={out:?}");
    let parts: Vec<&str> = out.trim().split(' ').collect();
    assert_eq!(parts.len(), 6, "unexpected buffer debug format: {out}");
    let sc: i32 = parts[0].parse().expect("sc parse");
    let buf: i32 = parts[1].parse().expect("buf parse");
    let len: i32 = parts[2].parse().expect("len parse");
    let b0: i32 = parts[3].parse().expect("b0 parse");
    let b1: i32 = parts[4].parse().expect("b1 parse");
    let b2: i32 = parts[5].parse().expect("b2 parse");
    assert!(sc > 0 && buf > 0, "pointers should be non-zero: {out}");
    assert!(len > 0, "input should be read: {out}");
    assert_eq!(b0, 49, "expected '1' at first byte: {out}");
    assert_eq!(b1, 48, "expected '0' at second byte: {out}");
    assert_eq!(b2, 32, "expected space at third byte: {out}");
}

#[test]
fn wasi_fd_read_raw_iovec_debug() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "core/mem" as *
#import "std/stdio" as *

fn main <()*>()> ():
    let cap <i32> 64;
    let buf <i32> alloc cap;
    let iov <i32> alloc 8;
    let nread <i32> alloc 4;

    store_i32 iov buf;
    store_i32 add iov 4 cap;
    store_i32 nread 0;

    let errno <i32> fd_read 0 iov 1 nread;
    let n <i32> load_i32 nread;

    print_i32 errno;
    print " ";
    print_i32 n;
    print " ";
    if lt 0 n:
        then:
            let b0 <i32> load_u8 buf;
            print_i32 b0
        else print_i32 -1;
    print " ";
    if lt 1 n:
        then:
            let b1 <i32> load_u8 add buf 1;
            print_i32 b1
        else print_i32 -1;
    print " ";
    if lt 2 n:
        then:
            let b2 <i32> load_u8 add buf 2;
            print_i32 b2
        else print_i32 -1;
    println "";
"#;
    let out = run_main_capture_stdout_with_stdin(src, b"10 20 30\n");
    let parts: Vec<&str> = out.trim().split(' ').collect();
    assert_eq!(parts.len(), 5, "unexpected raw fd_read format: {out}");
    let errno: i32 = parts[0].parse().expect("errno parse");
    let n: i32 = parts[1].parse().expect("n parse");
    let b0: i32 = parts[2].parse().expect("b0 parse");
    let b1: i32 = parts[3].parse().expect("b1 parse");
    let b2: i32 = parts[4].parse().expect("b2 parse");
    assert_eq!(errno, 0, "fd_read errno should be 0: {out}");
    assert!(n > 0, "fd_read should read bytes: {out}");
    assert_eq!(b0, 49, "expected '1' at first byte: {out}");
    assert_eq!(b1, 48, "expected '0' at second byte: {out}");
    assert_eq!(b2, 32, "expected space at third byte: {out}");
}

#[test]
fn wasi_fd_read_raw_iovec_with_dealloc_debug() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "core/mem" as *
#import "std/stdio" as *

fn main <()*>()> ():
    let cap <i32> 64;
    let buf <i32> alloc cap;
    let iov <i32> alloc 8;
    let nread <i32> alloc 4;

    store_i32 iov buf;
    store_i32 add iov 4 cap;
    store_i32 nread 0;

    let errno <i32> fd_read 0 iov 1 nread;
    let n <i32> load_i32 nread;
    dealloc iov 8;
    dealloc nread 4;

    print_i32 errno;
    print " ";
    print_i32 n;
    print " ";
    if lt 0 n:
        then:
            let b0 <i32> load_u8 buf;
            print_i32 b0
        else print_i32 -1;
    print " ";
    if lt 1 n:
        then:
            let b1 <i32> load_u8 add buf 1;
            print_i32 b1
        else print_i32 -1;
    print " ";
    if lt 2 n:
        then:
            let b2 <i32> load_u8 add buf 2;
            print_i32 b2
        else print_i32 -1;
    println "";
"#;
    let out = run_main_capture_stdout_with_stdin(src, b"10 20 30\n");
    let parts: Vec<&str> = out.trim().split(' ').collect();
    assert_eq!(parts.len(), 5, "unexpected raw fd_read format: {out}");
    let errno: i32 = parts[0].parse().expect("errno parse");
    let n: i32 = parts[1].parse().expect("n parse");
    let b0: i32 = parts[2].parse().expect("b0 parse");
    let b1: i32 = parts[3].parse().expect("b1 parse");
    let b2: i32 = parts[4].parse().expect("b2 parse");
    assert_eq!(errno, 0, "fd_read errno should be 0: {out}");
    assert!(n > 0, "fd_read should read bytes: {out}");
    assert_eq!(b0, 49, "expected '1' at first byte: {out}");
    assert_eq!(b1, 48, "expected '0' at second byte: {out}");
    assert_eq!(b2, 32, "expected space at third byte: {out}");
}

#[test]
fn wasi_fd_read_then_alloc_header_debug() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "core/mem" as *
#import "std/stdio" as *

fn main <()*>()> ():
    let cap <i32> 64;
    let buf <i32> alloc cap;
    let iov <i32> alloc 8;
    let nread <i32> alloc 4;

    store_i32 iov buf;
    store_i32 add iov 4 cap;
    store_i32 nread 0;
    let errno <i32> fd_read 0 iov 1 nread;
    let n <i32> load_i32 nread;

    dealloc iov 8;
    dealloc nread 4;

    let sc <i32> alloc 12;
    store_i32 sc buf;
    store_i32 add sc 4 n;
    store_i32 add sc 8 0;

    print_i32 errno;
    print " ";
    print_i32 n;
    print " ";
    print_i32 sc;
    print " ";
    print_i32 buf;
    print " ";
    if lt 0 n:
        then:
            let b0 <i32> load_u8 buf;
            print_i32 b0
        else print_i32 -1;
    print " ";
    if lt 1 n:
        then:
            let b1 <i32> load_u8 add buf 1;
            print_i32 b1
        else print_i32 -1;
    print " ";
    if lt 2 n:
        then:
            let b2 <i32> load_u8 add buf 2;
            print_i32 b2
        else print_i32 -1;
    println "";
"#;
    let out = run_main_capture_stdout_with_stdin(src, b"10 20 30\n");
    let parts: Vec<&str> = out.trim().split(' ').collect();
    assert_eq!(parts.len(), 7, "unexpected raw alloc-header format: {out}");
    let errno: i32 = parts[0].parse().expect("errno parse");
    let n: i32 = parts[1].parse().expect("n parse");
    let sc: i32 = parts[2].parse().expect("sc parse");
    let buf: i32 = parts[3].parse().expect("buf parse");
    let b0: i32 = parts[4].parse().expect("b0 parse");
    let b1: i32 = parts[5].parse().expect("b1 parse");
    let b2: i32 = parts[6].parse().expect("b2 parse");
    assert_eq!(errno, 0, "fd_read errno should be 0: {out}");
    assert!(n > 0, "fd_read should read bytes: {out}");
    assert!(sc > 0 && buf > 0, "pointers should be non-zero: {out}");
    assert_eq!(b0, 49, "expected '1' at first byte: {out}");
    assert_eq!(b1, 48, "expected '0' at second byte: {out}");
    assert_eq!(b2, 32, "expected space at third byte: {out}");
}

#[test]
fn local_scanner_new_logic_debug() {
    let src = r#"
#entry main
#indent 4
#target wasi

#import "core/mem" as *
#import "std/stdio" as *

fn scanner_new_local <()*>i32> ():
    let mut cap <i32> 65536;
    let mut buf <i32> alloc cap;
    if:
        eq buf 0
        then:
            let sc0 <i32> alloc 12;
            store_i32 sc0 0;
            store_i32 add sc0 4 0;
            store_i32 add sc0 8 0;
            sc0
        else:
            let iov <i32> alloc 8;
            let nread_ptr <i32> alloc 4;
            let mut len <i32> 0;
            let mut done <i32> 0;

            while eq done 0:
                do:
                    if:
                        eq len cap
                        then:
                            let new_cap <i32> mul cap 2;
                            let new_buf <i32> realloc buf cap new_cap;
                            if:
                                eq new_buf 0
                                then:
                                    set done 1;
                                else:
                                    set buf new_buf;
                                    set cap new_cap;
                        else:
                            ();

                    if:
                        eq done 0
                        then:
                            let write_ptr <i32> add buf len;
                            let rem <i32> sub cap len;
                            store_i32 iov write_ptr;
                            store_i32 add iov 4 rem;
                            store_i32 nread_ptr 0;
                            let errno <i32> fd_read 0 iov 1 nread_ptr;
                            if:
                                ne errno 0
                                then:
                                    set done 1;
                                else:
                                    let n <i32> load_i32 nread_ptr;
                                    if:
                                        eq n 0
                                        then:
                                            set done 1;
                                        else:
                                            set len add len n;
                        else:
                            ();

            dealloc iov 8;
            dealloc nread_ptr 4;

            let sc <i32> alloc 12;
            store_i32 sc buf;
            store_i32 add sc 4 len;
            store_i32 add sc 8 0;
            sc

fn main <()*>()> ():
    let sc <i32> scanner_new_local;
    let buf <i32> load_i32 sc;
    let len <i32> load_i32 add sc 4;
    let b0 <i32> if lt 0 len load_u8 buf -1;
    let b1 <i32> if lt 1 len load_u8 add buf 1 -1;
    let b2 <i32> if lt 2 len load_u8 add buf 2 -1;
    print_i32 len;
    print " ";
    print_i32 b0;
    print " ";
    print_i32 b1;
    print " ";
    print_i32 b2;
    println "";
"#;
    let out = run_main_capture_stdout_with_stdin(src, b"10 20 30\n");
    let parts: Vec<&str> = out.trim().split(' ').collect();
    assert_eq!(parts.len(), 4, "unexpected local scanner format: {out}");
    let len: i32 = parts[0].parse().expect("len parse");
    let b0: i32 = parts[1].parse().expect("b0 parse");
    let b1: i32 = parts[2].parse().expect("b1 parse");
    let b2: i32 = parts[3].parse().expect("b2 parse");
    assert!(len > 0, "input should be read: {out}");
    assert_eq!(b0, 49, "expected '1' at first byte: {out}");
    assert_eq!(b1, 48, "expected '0' at second byte: {out}");
    assert_eq!(b2, 32, "expected space at third byte: {out}");
}
