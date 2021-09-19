use crate::ast;

// A customized result type
#[must_use]
#[derive(Clone, Debug)]
pub struct Outcome<A> {
    ok: Option<A>,
    errors: Vec<String>,
    warnings: Vec<String>,
}

impl<A> Outcome<A> {
    fn new_empty() -> Outcome<A> {
        Outcome {
            ok: None,
            errors: vec![],
            warnings: vec![],
        }    
    }

    fn new(a: A) -> Outcome<A> {
        Outcome {
            ok: Some(a),
            errors: vec![],
            warnings: vec![],
        }
    }

    fn new_err(s: String) -> Outcome<A> {
        Outcome {
            ok: None,
            errors: vec![s],
            warnings: vec![],
        }
    }

    fn new_warn(s: String) -> Outcome<A> {
        Outcome {
            ok: None,
            errors: vec![s],
            warnings: vec![],
        }
    }

    fn set_ok(self, a: A) -> Outcome<A> {
        Outcome { ok: Some(a), errors: self.errors, warnings: self.warnings}
    }

    fn map<B>(self, f: impl FnOnce(A) -> B) -> Outcome<B> {
        match self.ok {
            Some(t) => Outcome { ok: Some(f(t)), errors: self.errors, warnings: self.warnings },
            None => Outcome { ok: None, errors: self.errors, warnings: self.warnings },
        }
    }

    fn and_then<B>(mut self, f: impl FnOnce(A) -> Outcome<B>) -> Outcome<B> {
        match self.ok {
            Some(o) => {
                let mut outcome = f(o);
                self.errors.append(&mut outcome.errors);
                self.warnings.append(&mut outcome.warnings);

                Outcome { ok: outcome.ok, errors: self.errors, warnings: self.warnings }
            },
            None => Outcome { ok: None, errors: self.errors, warnings: self.warnings },
        }
    }

    fn and_zip<B>(mut self, mut other: Outcome<B>) -> Outcome<(A, B)> {
        self.errors.append(&mut other.errors);
        self.warnings.append(&mut other.warnings);

        match (self.ok, other.ok) {
            (Some(l), Some(r)) => Outcome { ok: Some((l, r)), errors: self.errors, warnings: self.warnings },
            (_, _) => Outcome { ok: None, errors: self.errors, warnings: self.warnings },
        }
    }
}

fn infer(e: &ast::SpanExpr) -> Outcome<ast::JustType> {
    match e {
        ast::Expr::BoolLit(_, _) => Outcome::new(ast::Type::Bool(())),
        ast::Expr::NatLit(_, _) => Outcome::new(ast::Type::Nat(())),
        ast::Expr::TypeAnno { term, ty, .. } => {
            infer(term).and_then(|term_ty| {
                    println!("||||{} == {}", term, ty);
                    if term_ty == ty.strip() {
                        Outcome::new_empty()
                    } else {
                        Outcome::new_err(format!("found `{}` type, but expected `{}` type", term_ty, ty))
                    }
                })
                .set_ok(ty.strip())
        },
        ast::Expr::IfFlow { cond, on_true, on_false, .. } => {
            infer(cond)
                .and_then(|ty| {
                    if ty.strip() == ast::Type::Bool(()) {
                        match **cond {
                            ast::Expr::BoolLit(_, b) => Outcome::new_warn(format!("`if` condition is always `{}`", b)),
                            _ => Outcome::new_empty(),
                        }
                    } else {
                        Outcome::new_err(format!("the type of condition in `if` has to be `bool`, but is `{}`", ty))
                    }
                })
                .set_ok(ast::Type::Bool(()))
                .and_then(|_| infer(on_true).and_zip(infer(on_false)))
                .and_then(|(first, second)| {
                    if first == second {
                        Outcome::new(first)
                    } else {
                        Outcome::new_err(format!(
                            "branches of `if` must be the same, `{}` does not equal `{}`",
                            first,
                            second
                        ))
                    }
                })
        },
        ast::Expr::Error => panic!("Internal compiler error."),
    }
}