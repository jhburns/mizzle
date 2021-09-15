extern crate lalrpop;

fn main() {
    // Converts all lalrpop files into Rust ones
    lalrpop::process_root().unwrap();
}