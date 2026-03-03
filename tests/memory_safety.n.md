# memory safety 回帰テスト

## alloc_ptr/load_store/dealloc_ptr の基本動作

neplg2:test
ret: 123
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/result" as *

fn main <()->i32> ():
    match alloc_ptr<i32> 4:
        Result::Err _e:
            0
        Result::Ok p:
            match store_i32_ptr p 123:
                Result::Err _e:
                    0
                Result::Ok _:
                    let v <i32> match load_i32_ptr p:
                        Option::None:
                            0
                        Option::Some x:
                            x
                    match dealloc_ptr p 4:
                        Result::Err _e:
                            0
                        Result::Ok _:
                            v
```

## 無効ポインタ load は Option::None

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/option" as *

fn main <()->i32> ():
    let p <MemPtr<i32>> mem_ptr_wrap 0
    match load_i32_ptr p:
        Option::None:
            1
        Option::Some _v:
            0
```

## 無効ポインタ store は Result::Err

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/result" as *

fn main <()->i32> ():
    let p <MemPtr<i32>> mem_ptr_wrap 0
    match store_i32_ptr p 42:
        Result::Err _e:
            1
        Result::Ok _:
            0
```
