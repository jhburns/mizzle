use lalrpop_util::{ParseError};
use lalrpop_util::lexer::{Token};

use colored::*;

use crate::type_check;

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

#[derive(Debug, Clone)]
enum AccentColor {
    Error,
    Warning,
}

fn format_source(source: &Vec<String>, l1: usize, l2: Option<usize>, color: AccentColor) -> String {
    let fmt_accent = |s: String| -> ColoredString {
        match &color {
            AccentColor::Error => s.bright_red(),
            AccentColor::Warning => s.truecolor(255, 165, 0),
        }
    };

    let p1 = loc_to_pnt(source, l1);
    let p2 = l2.map(|l| loc_to_pnt(source, l)).unwrap_or(p1);


    let line_number = format!("{} |", p1.0).bright_blue();
    let indicator_offset = line_number.len() + p1.1;

    // Assuming that the points are always on the same line
    format!("{}{}\n{}{}{}{}",
        line_number,
        source[p1.0],
        " ".repeat(indicator_offset),
        fmt_accent("^".into()),
        fmt_accent("^".repeat(if p2.1 - p1.1 == 0 { 0 } else { p2.1 - p1.1 - 1 })),
        fmt_accent("here".into())
    )
}

pub fn format_parse_err(err: ParseError<usize, Token<'_>, &'static str>, source: &Vec<String>) -> String {
    let prefix = format!("{}: ", "Parse error".bright_red());
    
    match err {
        ParseError::InvalidToken { location } => {
            format!(
                "{}illegal character(s).\n{}",
                prefix,
                format_source(source, location, None, AccentColor::Error)
            )
        },
        ParseError::UnrecognizedEOF { location, expected } => {
            format!(
                "{}file ended, but expected {}.\n{}",
                prefix,
                format_expected(expected),
                format_source(source, location, None, AccentColor::Error)
            )
        },
        ParseError::UnrecognizedToken { token: (l_start, token, l_end), expected } => {
            format!(
                "{}`{}` is unexpected, expected {}.\n{}",
                prefix,
                token,
                format_expected(expected),
                format_source(source, l_start, Some(l_end), AccentColor::Error)
            )
        },
        ParseError::ExtraToken { token: (l_start, token, l_end) } => {
            format!(
                "{}extra `{}`.\n{}",
                prefix,
                token,
                format_source(source, l_start, Some(l_end), AccentColor::Error)
            )
        },
        ParseError::User { error } => format!("{}{}.", prefix, error),
    }
}

pub fn format_type_err(e: type_check::TypeError, source: &Vec<String>) -> String {
    let prefix = format!("{}: ", "Type error".bright_red());

    match e {
        type_check::TypeError::AnnotationIncorrect { span, got, annotation } => {
            format!(
                "{}type is `{}`, but annotation is `{}`.\n{}",
                prefix,
                got,
                annotation,
                format_source(source, span.0, Some(span.1), AccentColor::Error)
            )
        },
        type_check::TypeError::IfCondMustBeBool { end, got } => {
            format!(
                "{}the condition of `if` should be `bool`, but is `{}`.\n{}",
                prefix,
                got,
                format_source(source, end - 1, None, AccentColor::Error)
            )
        },
        type_check::TypeError::IfBranchesMustBeSame { start, first, second } => {
            format!(
                "{}branches of `if` have to return the same type, `{}` is not equal to `{}`.\n{}",
                prefix,
                first,
                second,
                format_source(source, start, None, AccentColor::Error)
            )
        },
    }
}

pub fn format_type_warn(w: type_check::TypeWarning, source: &Vec<String>) -> String {
    let prefix = format!("{}: ", "Warning".truecolor(255, 165, 0));

    match w {
        type_check::TypeWarning::CondAlways { span, value } => {
            format!(
                "{}condition of `if` is always `{}`.\n{}",
                prefix,
                value,
                format_source(source, span.0, Some(span.1), AccentColor::Warning)
            )
        }
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