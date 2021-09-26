# mizzle

An inconsequential programming language.

mizzle is created with:
- A [LALRPOP](https://crates.io/crates/lalrpop) lexer and parser
- Advanced type checking inspired by this post: https://keleshev.com/advanced-error-handling-in-ocaml
- [parity-wasm](https://crates.io/crates/parity-wasm) used for backend code generation
- [wasmer](https://crates.io/crates/wasmer/) with the [cranelift compiler](https://crates.io/crates/wasmer-compiler-cranelift) as the runtime