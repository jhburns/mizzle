use std::fmt::{Debug, Display, Error, Formatter};
use lalrpop_util::{ParseError};
use lalrpop_util::lexer::{Token};
use colored::*;

// LALRPOP is setup to parse into Expr,
// `Debug` trait implemented manually to look better

#[derive(Clone, Copy, Debug)]
pub enum Type {
    Nat,
    Bool,
}

impl Display for Type {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Type::Nat => write!(fmt, "nat"),
            Type::Bool => write!(fmt, "bool"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Expr {
    NatLit(u64),
    BoolLit(bool),
    TypeAnno { term: Box<Expr>, ty: Type },
    IfFlow { cond: Box<Expr>, on_true: Box<Expr>, on_false: Box<Expr> },
    Error,
}


fn pretty_expr(e: &Expr, indent: usize) -> String {
    match e {
        Expr::NatLit(n) => n.to_string(),
        Expr::BoolLit(b) => b.to_string(),
        Expr::TypeAnno { term, ty } => format!("{}: {}", pretty_expr(term, indent), ty.to_string()),
        Expr::IfFlow { cond, on_true, on_false } => {
            let indents = "\t".repeat(indent);
            let pretty_cond = pretty_expr(cond, indent);
            let pretty_on_true = pretty_expr(on_true, indent + 1);
            let pretty_on_false = pretty_expr(on_false, indent + 1);

            format!(r#"if {1} then
{0}	{2}
{0}else
{0}	{3}
{0}end"#, indents, pretty_cond, pretty_on_true, pretty_on_false)
        },
        Expr::Error => "error".into(),
    }
}

impl Display for Expr {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{}", pretty_expr(&self, 0))
    }
}

// TODO: add "or" for multiple expected tokens
// Functions for formatting parser errors
fn format_expected(expected: Vec<String>) -> String {
    format!(
        "{}{}",
        if expected.len() <= 1 { "" } else { "one of " },
        expected.iter().map(|s| {
                // Remove first and last character because token are wrapped in parentheses
                let mut chars = s.chars();
                chars.next();
                chars.next_back();
                chars.as_str()    
            }).map(|s| format!("`{}`", s))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn loc_to_pnt(source: &Vec<String>, mut l: usize) -> (usize, usize) {
    let line_lengths = source.iter().map(|s| s.len()).collect::<Vec<_>>();

    let mut i = 0;
    loop {
        // If we reach the end of the file,
        // Return a point one after the last character of the last line
        if i > line_lengths.len() - 1 {
            i = line_lengths.len() - 1;
            l = line_lengths[line_lengths.len() - 1];
            break
        }

        if l <= line_lengths[i] {
            break 
        }

        // `+ 1` because the newline character is removed when split
        l -= line_lengths[i] + 1;
        i += 1;
    }

    (i, l)
}

fn format_source(source: &Vec<String>, l1: usize, l2: Option<usize>) -> String {
    let p1 = loc_to_pnt(source, l1);
    let p2 = l2.map(|l| loc_to_pnt(source, l)).unwrap_or(p1);


    let line_number = format!("{} |", p1.0).bright_blue();
    let indicator_offset = line_number.len() + p1.1;

    // Assuming that the points are always on the same line
    format!("{}{}\n{}{}{}{}",
        line_number,
        source[p1.0],
        " ".repeat(indicator_offset),
        "^".bright_red(),
        "^".repeat(if p2.1 - p1.1 == 0 { 0 } else { p2.1 - p1.1 - 1 }).bright_red(),
        "here".bright_red()
    )
}

pub fn format_parse_err(err: ParseError<usize, Token<'_>, &'static str>, source: &Vec<String>) -> String {
    let error_start = format!("{}: ", "Parse error".bright_red());
    
    match err {
        ParseError::InvalidToken { location } => {
            format!(
                "{}illegal character(s).\n{}",
                error_start,
                format_source(source, location, None)
            )
        },
        ParseError::UnrecognizedEOF { location, expected } => {
            format!(
                "{}file ended, but expected {}.\n{}",
                error_start,
                format_expected(expected),
                format_source(source, location, None)
            )
        },
        ParseError::UnrecognizedToken { token: (l_start, token, l_end), expected } => {
            format!(
                "{}`{}` is unexpected, expected {}.\n{}",
                error_start,
                token,
                format_expected(expected),
                format_source(source, l_start, Some(l_end))
            )
        },
        ParseError::ExtraToken { token: (l_start, token, l_end) } => {
            format!(
                "{}extra `{}`.\n{}",
                error_start,
                token,
                format_source(source, l_start, Some(l_end))
            )
        },
        ParseError::User { error } => format!("{}{}.", error_start, error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn location_coversion() {
        let source =
r#"abc
df"#.split("\n").map(|s| s.into()).collect::<Vec<String>>();

        assert_eq!(loc_to_pnt(&source, 0), (0, 0));
        assert_eq!(loc_to_pnt(&source, 2), (0, 2));
        assert_eq!(loc_to_pnt(&source, 4), (1, 0));
        assert_eq!(loc_to_pnt(&source, 6), (1, 2));
        assert_eq!(loc_to_pnt(&source, 20), (1, 2));
    }
}