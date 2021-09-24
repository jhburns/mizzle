use crate::ast;

#[derive(Clone, Debug)]
pub enum TypeError {
    AnnotationIncorrect { span: ast::Span, got: ast::JustType, annotation: ast::JustType },
    IfCondMustBeBool { end: usize, got: ast::JustType },
    IfBranchesMustBeSame { start: usize, first: ast::JustType, second: ast::JustType },
}

#[derive(Clone, Debug)]
pub enum TypeWarning {
    CondAlways { span: ast::Span, value: bool }
}

// A customized result type
#[must_use]
#[derive(Clone, Debug)]
pub struct Outcome<A> {
    pub ok: Option<A>,
    pub errors: Vec<TypeError>,
    pub warnings: Vec<TypeWarning>,
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

    fn new_err(e: TypeError) -> Outcome<A> {
        Outcome {
            ok: None,
            errors: vec![e],
            warnings: vec![],
        }
    }

    fn new_warn(w: TypeWarning) -> Outcome<A> {
        Outcome {
            ok: None,
            errors: vec![],
            warnings: vec![w],
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

pub fn infer(e: &ast::SpanExpr) -> Outcome<ast::JustType> {
    match e {
        ast::Expr::BoolLit(_, _) => Outcome::new(ast::Type::Bool(())),
        ast::Expr::NatLit(_, _) => Outcome::new(ast::Type::Nat(())),
        ast::Expr::TypeAnno { term, ty, .. } => {
            infer(term).and_then(|term_ty| {
                    if term_ty == ty.strip() {
                        Outcome::new_empty()
                    } else {
                        Outcome::new_err(TypeError::AnnotationIncorrect {
                            span: *ty.extra(),
                            got: term_ty,
                            annotation: ty.strip(),
                        })
                    }
                })
                .set_ok(ty.strip())
        },
        ast::Expr::IfFlow { cond, on_true, on_false, extra } => {
            infer(cond)
                .and_then(|ty| {
                    if ty.strip() == ast::Type::Bool(()) {
                        match **cond {
                            ast::Expr::BoolLit(span, b) => Outcome::new_warn(TypeWarning::CondAlways {
                                span,
                                value: b 
                            }),
                            _ => Outcome::new_empty(),
                        }
                    } else {
                        Outcome::new_err(TypeError::IfCondMustBeBool { end: cond.extra().1, got: ty })
                    }
                })
                .set_ok(ast::Type::Bool(()))
                .and_then(|_| infer(on_true).and_zip(infer(on_false)))
                .and_then(|(first, second)| {
                    if first == second {
                        Outcome::new(first)
                    } else {
                        Outcome::new_err(TypeError::IfBranchesMustBeSame { start: extra.0, first, second })
                    }
                })
        },
        ast::Expr::Error => panic!("Internal compiler error."),
    }
}