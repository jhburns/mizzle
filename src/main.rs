mod syntax_test;
mod ast;
mod error_fmt;
mod type_check;

#[macro_use] extern crate lalrpop_util;

// Synthesized by LALRPOP
lalrpop_mod!(pub syntax);

fn main() {
    println!("Hello, world!");
}
