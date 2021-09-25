use std::fmt::{Debug, Display, Error, Formatter};

// LALRPOP is setup to parse into Expr,
// `Debug` trait implemented manually to look better

#[derive(Clone, Copy, Debug)]
pub enum Type<T> {
    Nat(T),
    Bool(T),
}

pub type JustType = Type<()>;

impl<T> Type<T> {
    pub fn strip(&self) -> JustType {
        match self {
            Type::Nat(_) => Type::Nat(()),
            Type::Bool(_) => Type::Bool(()),
        }
    }

    pub fn extra(&self) -> &T {
        match self {
            Type::Nat(e) => e,
            Type::Bool(e) => e,
        }
    }

    pub fn map_extra<U>(&self, f: &Fn(&T) -> U) -> Type<U> {
        match self {
            Type::Nat(extra) => Type::Nat(f(extra)),
            Type::Bool(extra) => Type::Bool(f(extra)),
        }
    }
}

impl<T> PartialEq for Type<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Nat(_), Self::Nat(_)) => true,
            (Self::Bool(_), Self::Bool(_)) => true,
            _ => false,
        }
    }
}

impl<T> Eq for Type<T> {}

impl<T> Display for Type<T> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Type::Nat(_) => write!(fmt, "nat"),
            Type::Bool(_) => write!(fmt, "bool"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Expr<T> {
    NatLit(T, u64),
    BoolLit(T, bool),
    TypeAnno { extra: T, term: Box<Expr<T>>, ty: Type<T> },
    IfFlow { extra: T, cond: Box<Expr<T>>, on_true: Box<Expr<T>>, on_false: Box<Expr<T>> },
    Error,
}

impl<T> Expr<T> {
    pub fn extra(&self) -> &T {
        match self {
            Expr::NatLit(extra, _) => extra,
            Expr::BoolLit(extra, _) => extra,
            Expr::TypeAnno { extra, .. } => extra,
            Expr::IfFlow { extra, .. } => extra,
            Expr::Error => panic!("Internal compiler error"),
        }
    }

    pub fn map_extra<U>(&self, f: &Fn(&T) -> U) -> Expr<U> {
        match self {
            Expr::NatLit(extra, n) => Expr::NatLit(f(extra), *n),
            Expr::BoolLit(extra, b) => Expr::BoolLit(f(extra), *b),
            Expr::TypeAnno { extra, term, ty } => Expr::TypeAnno {
                extra: f(extra),
                term: Box::new(term.map_extra(f)),
                ty: ty.map_extra(f)
            },
            Expr::IfFlow { extra,  cond, on_true, on_false } => Expr::IfFlow {
                extra: f(extra),
                cond: Box::new(cond.map_extra(f)),
                on_true: Box::new(on_true.map_extra(f)),
                on_false: Box::new(on_false.map_extra(f)),
            },
            Expr::Error => panic!("Internal compiler error"),
        }
    }
 }

pub type JustExpr = Expr<()>;

fn pretty_expr<T>(e: &Expr<T>, indent: usize) -> String {
    match e {
        Expr::NatLit(_, n) => n.to_string(),
        Expr::BoolLit(_, b) => b.to_string(),
        Expr::TypeAnno { term, ty, .. } => format!("{}: {}", pretty_expr(term, indent), ty.to_string()),
        Expr::IfFlow { cond, on_true, on_false, .. } => {
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

impl<T> Display for Expr<T> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{}", pretty_expr(self, 0))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Span(pub usize, pub usize);

pub type SpanType = Type<Span>;
pub type SpanExpr = Expr<Span>;