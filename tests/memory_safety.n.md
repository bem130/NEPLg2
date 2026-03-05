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

## alloc/dealloc の基本動作

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/result" as *

fn main <()->i32> ():
    match alloc 8:
        Result::Err _e:
            0
        Result::Ok p:
            store_i32 p 77
            let ok <i32> if eq load_i32 p 77 1 0
            match dealloc p 8:
                Result::Err _e:
                    0
                Result::Ok _:
                    ok
```

## dealloc は無効引数を Err で返す

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/result" as *

fn main <()->i32> ():
    match dealloc 0 4:
        Result::Err _e:
            1
        Result::Ok _:
            0
```

## alloc_region/region_ptr_at/dealloc_region の基本動作

neplg2:test
ret: 321
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/result" as *
#import "core/option" as *

fn main <()->i32> ():
    match alloc_region<i32> 1:
        Result::Err _e:
            0
        Result::Ok token:
            match region_ptr_at<i32,i32> token 0:
                Result::Err _e:
                    0
                Result::Ok p:
                    match store_i32 p 321:
                        Result::Err _e:
                            0
                        Result::Ok _:
                            let v <i32> match load_i32 p:
                                Option::None:
                                    0
                                Option::Some x:
                                    x
                            match dealloc_region token:
                                Result::Err _e:
                                    0
                                Result::Ok _:
                                    v
```

## region_ptr_at は範囲外アクセスを Err で返す

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/result" as *

fn main <()->i32> ():
    match alloc_region<i32> 1:
        Result::Err _e:
            0
        Result::Ok token:
            let out <Result<MemPtr<i32>,str>> region_ptr_at<i32,i32> token 4
            let ok <i32> match out:
                Result::Err _e:
                    1
                Result::Ok _:
                    0
            match dealloc_region token:
                Result::Err _e:
                    0
                Result::Ok _:
                    ok
```
