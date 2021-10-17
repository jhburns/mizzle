mod ast;
mod error_fmt;
mod syntax_test;
mod type_check;
mod wasm;

#[macro_use]
extern crate lalrpop_util;

use std::env;
use std::fs;

use crate::syntax::TermParser;

// Synthesized by LALRPOP
lalrpop_mod!(pub syntax);

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Please supply a filename to run. Like `$ mizzle ok.mi`");
        return Ok(());
    }

    let filename = &args[1];

    let source = fs::read_to_string(filename)?;
    let source_lines = source
        .split('\n')
        .map(|s| s.into())
        .collect::<Vec<String>>();

    match TermParser::new().parse(&source) {
        Ok(a) => {
            let check_result = type_check::check(&a);

            let mut issues = vec![];

            let mut warn_issues = check_result
                .warnings
                .into_iter()
                .map(type_check::TypeIssue::Warning)
                .collect::<Vec<_>>();

            issues.append(&mut warn_issues);

            match check_result.result {
                Ok(final_ty) => {
                    issues.sort();

                    for issue in issues {
                        print!("{}\n\n", error_fmt::format_type_issue(issue, &source_lines));
                    }

                    wasm::eval(wasm::ast_to_wasm(&a.map_extra(&|_| ())), &final_ty)
                }
                Err(errors) => {
                    let mut err_issues = errors
                        .into_iter()
                        .map(type_check::TypeIssue::Error)
                        .collect::<Vec<_>>();

                    issues.append(&mut err_issues);
                    issues.sort();

                    for issue in issues {
                        print!("{}\n\n", error_fmt::format_type_issue(issue, &source_lines));
                    }
                }
            }
        }
        Err(e) => println!("{}\n", error_fmt::format_parse_err(e, &source_lines)),
    }

    Ok(())
}
