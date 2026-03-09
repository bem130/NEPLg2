# ret_string_example

`ret:` に JSON 文字列（"..."）を指定し、`main` の戻り値（i32 ポインタ）を NEPL の `str` 表現（[len:u32][bytes...]）として復号して比較します。

## return_str

neplg2:test
ret: "hello"
```neplg2
#entry main
#indent 4

fn main <()->str>():
    "hello"
```
