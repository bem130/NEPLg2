# NEPLG2

Prefix + off-side rule language that targets WebAssembly only. All operators, control forms, and type annotations are prefix; blocks are introduced with `:` and indentation. The repository ships a minimal core compiler (`nepl-core`) and a CLI (`nepl-cli`) that compiles to WASM and can run it via wasmi.

## Language in 30 seconds
- Prefix everything; function names alone are not expressions.
- Off-side rule blocks: put `:` at line end, indent the next line; the block closes when indent returns. A block is an expression.
- `;` is allowed only directly under a `:` block and pops the last value (stack restore). Without `;`, the block’s last value is its result.
- Effects: `a->b` is pure; `a*>b` is impure. Pure code cannot call impure code.
- Unit is `()`; `<T>` is an identity annotation treated like a prefix operator.

Example:
```neplg2
#entry main
#indent 4

#import "std/stdio"
#use std::stdio::*

fn main <()*> ()> ():
    let mut x <i32> 0;
    while lt x 5:
        <()> print_i32 x;
        set x add x 1;
    ()
```

## CLI
Compile and run:
```bash
cargo run -p nepl-cli -- --input examples/counter.nepl --output target/counter.wasm --run
```
`#import` resolves relative to the source file; `std/*` paths come from `./stdlib`. `#use` is handled after inlining imports.

## Standard library
- `stdlib/std.nepl`: basic arithmetic/comparison ops (`add`, `sub`, `lt`) implemented with `#wasm`.
- `stdlib/std/stdio.nepl`: placeholder module; the runtime provides host import `env.print_i32 : (i32)*>()`.

## Examples
- `examples/counter.nepl` – print 0..4.
- `examples/fib.nepl` – print the first ten Fibonacci numbers.

## Testing
Run all tests:
```bash
cargo test --workspace --locked
```
