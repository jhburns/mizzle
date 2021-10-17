use std::fmt::{Debug, Display, Error, Formatter};

// LALRPOP is setup to parse into Expr,
// `Display` trait implemented manually for pretty printing

#[derive(Clone, Copy, Debug)]
pub enum Type<T> {
    Int(T),
    Bool(T),
}

pub type JustType = Type<()>;

impl<T> Type<T> {
    pub fn strip(&self) -> JustType {
        match self {
            Type::Int(_) => Type::Int(()),
            Type::Bool(_) => Type::Bool(()),
        }
    }

    pub fn extra(&self) -> &T {
        match self {
            Type::Int(e) => e,
            Type::Bool(e) => e,
        }
    }

    pub fn map_extra<U>(&self, f: &dyn Fn(&T) -> U) -> Type<U> {
        match self {
            Type::Int(extra) => Type::Int(f(extra)),
            Type::Bool(extra) => Type::Bool(f(extra)),
        }
    }
}

impl<T> PartialEq for Type<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Int(_), Self::Int(_)) => true,
            (Self::Bool(_), Self::Bool(_)) => true,
            _ => false,
        }
    }
}

impl<T> Eq for Type<T> {}

impl<T> Display for Type<T> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Type::Int(_) => write!(fmt, "int"),
            Type::Bool(_) => write!(fmt, "bool"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Expr<T> {
    IntLit(T, i64),
    BoolLit(T, bool),
    TypeAnno {
        extra: T,
        term: Box<Expr<T>>,
        ty: Type<T>,
    },
    IfFlow {
        extra: T,
        cond: Box<Expr<T>>,
        on_true: Box<Expr<T>>,
        on_false: Box<Expr<T>>,
    },
}

impl<T> Expr<T> {
    pub fn extra(&self) -> &T {
        match self {
            Expr::IntLit(extra, _) => extra,
            Expr::BoolLit(extra, _) => extra,
            Expr::TypeAnno { extra, .. } => extra,
            Expr::IfFlow { extra, .. } => extra,
        }
    }

    pub fn map_extra<U>(&self, f: &dyn Fn(&T) -> U) -> Expr<U> {
        match self {
            Expr::IntLit(extra, n) => Expr::IntLit(f(extra), *n),
            Expr::BoolLit(extra, b) => Expr::BoolLit(f(extra), *b),
            Expr::TypeAnno { extra, term, ty } => Expr::TypeAnno {
                extra: f(extra),
                term: Box::new(term.map_extra(f)),
                ty: ty.map_extra(f),
            },
            Expr::IfFlow {
                extra,
                cond,
                on_true,
                on_false,
            } => Expr::IfFlow {
                extra: f(extra),
                cond: Box::new(cond.map_extra(f)),
                on_true: Box::new(on_true.map_extra(f)),
                on_false: Box::new(on_false.map_extra(f)),
            },
        }
    }
}

pub type JustExpr = Expr<()>;

fn pretty_expr<T>(e: &Expr<T>, indent: usize) -> String {
    match e {
        Expr::IntLit(_, n) => n.to_string(),
        Expr::BoolLit(_, b) => b.to_string(),
        Expr::TypeAnno { term, ty, .. } => {
            format!("{}: {}", pretty_expr(term, indent), ty.to_string())
        }
        Expr::IfFlow {
            cond,
            on_true,
            on_false,
            ..
        } => {
            let indents = "\t".repeat(indent);
            let pretty_cond = pretty_expr(cond, indent);
            let pretty_on_true = pretty_expr(on_true, indent + 1);
            let pretty_on_false = pretty_expr(on_false, indent + 1);

            format!(
                r#"if {1} then
{0}	{2}
{0}else
{0}	{3}
{0}end"#,
                indents, pretty_cond, pretty_on_true, pretty_on_false
            )
        }
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
