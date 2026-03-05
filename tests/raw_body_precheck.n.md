# raw body target precheck

## wasm_target_rejects_llvmir_body

neplg2:test[compile_fail]
diag_id: 3095
```neplg2
#target core
#entry main
#indent 4

fn main <()->i32> ():
    #llvmir:
        define i32 @main() {
        entry:
            ret i32 1
        }
```

## llvm_target_rejects_wasm_body

neplg2:test[llvm_cli, compile_fail]
diag_id: 3095
```neplg2
#target llvm
#entry main
#indent 4

fn main <()->i32> ():
    #wasm:
        i32.const 1
```

## active_raw_bodies_conflict_reports_diag

neplg2:test[compile_fail]
diag_id: 3094
```neplg2
#target core
#entry main
#indent 4

fn main <()->i32> ():
    #if[target=core]
    #wasm:
        i32.const 1
    #if[target=core]
    #llvmir:
        define i32 @main() {
        entry:
            ret i32 2
        }
```
