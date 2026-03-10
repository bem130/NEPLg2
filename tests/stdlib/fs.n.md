# fs facade

## fs_read_to_string_missing_file

このケースは、存在しないファイルを読んだときに `fs_read_to_string` が `Err` を返すことを確認します。
ファイルシステム依存の失敗を成功扱いしないことが目的です。

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "std/fs" as *
#import "std/test" as *

fn main <()*>i32> ():
    match fs_read_to_string "__definitely_missing_file__.txt":
        Result::Ok _s:
            test_fail "fs_read_to_string unexpectedly succeeded";
            1
        Result::Err _e:
            0
```

## fs_read_to_bytes_existing_file

このケースは、既知のテストファイルを `ByteBuf` として読み込み、その後 `str` に戻せることを確認します。
`std/fs` の binary path が `Vec<u8>` ではなく `ByteBuf` を返し、text 化経路も保たれていることが目的です。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "std/fs" as *
#import "std/test" as *

fn main <()*>i32> ():
    match fs_read_to_bytes "tests/stdlib/fs.n.md":
        Result::Ok bytes:
            let text <str> fs_bytes_to_string bytes;
            if:
                str_starts_with text "# fs facade"
                then 0
                else 2
        Result::Err _e:
            3
```
