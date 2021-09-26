# mizzle

An inconsequential programming language.

mizzle is created with:
- A [LALRPOP](https://crates.io/crates/lalrpop) lexer and parser
- Advanced type checking inspired by this post: https://keleshev.com/advanced-error-handling-in-ocaml
- [parity-wasm](https://crates.io/crates/parity-wasm) used for backend code generation
- [wasmer](https://crates.io/crates/wasmer/) with the [cranelift compiler](https://crates.io/crates/wasmer-compiler-cranelift) as the runtime

## Everything Possible

```
if true : bool then
    2
else
    3 : int
end
```

## Using

Install with `$ cargo install mizzle --path="/"` in this directory.

Run with `$ mizzle filename.mi`.