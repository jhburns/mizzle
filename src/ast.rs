use std::fmt::{Debug, Display, Error, Formatter};

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
        write!(fmt, "{}", pretty_expr(self, 0))
    }
}