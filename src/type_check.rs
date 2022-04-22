use crate::ast;
use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub enum TypeError {
    AnnotationIncorrect {
        span: ast::Span,
        got: ast::JustType,
        annotation: ast::JustType,
    },
    IfCondMustBeBool {
        end: usize,
        got: ast::JustType,
    },
    IfBranchesMustBeSame {
        start: usize,
        first: ast::JustType,
        second: ast::JustType,
    },
}

impl TypeError {
    fn first_location(&self) -> usize {
        match self {
            TypeError::AnnotationIncorrect { span, .. } => span.0,
            TypeError::IfCondMustBeBool { end, .. } => *end,
            TypeError::IfBranchesMustBeSame { start, .. } => *start,
        }
    }
}

#[derive(Clone, Debug)]
pub enum TypeWarning {
    CondAlways { span: ast::Span, value: bool },
}

impl TypeWarning {
    fn first_location(&self) -> usize {
        match self {
            TypeWarning::CondAlways { span, .. } => span.0,
        }
    }
}

#[derive(Clone, Debug)]
pub enum TypeIssue {
    Error(TypeError),
    Warning(TypeWarning),
}

impl TypeIssue {
    fn first_location(&self) -> usize {
        match self {
            TypeIssue::Error(e) => e.first_location(),
            TypeIssue::Warning(w) => w.first_location(),
        }
    }
}

impl PartialEq for TypeIssue {
    fn eq(&self, other: &Self) -> bool {
        self.first_location() == other.first_location()
    }
}

impl Eq for TypeIssue {}

impl PartialOrd for TypeIssue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.first_location().cmp(&other.first_location()))
    }
}

impl Ord for TypeIssue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.first_location().cmp(&other.first_location())
    }
}

// A similar type to `Result`
// Point of this type is to track errors that may occur
// And track the currently inferred type if it exists
#[must_use]
#[derive(Clone, Debug)]
struct Outcome<A> {
    pub result: Option<A>,
    pub errors: Vec<TypeError>,
    pub warnings: Vec<TypeWarning>,
}

impl<A> Outcome<A> {
    // Useful when an error needs to be returned in one branch but not the other
    fn new_empty() -> Outcome<A> {
        Outcome {
            result: None,
            errors: vec![],
            warnings: vec![],
        }
    }

    fn new(a: A) -> Outcome<A> {
        Outcome {
            result: Some(a),
            errors: vec![],
            warnings: vec![],
        }
    }

    fn new_err(e: TypeError) -> Outcome<A> {
        Outcome {
            result: None,
            errors: vec![e],
            warnings: vec![],
        }
    }

    fn new_warn(w: TypeWarning) -> Outcome<A> {
        Outcome {
            result: None,
            errors: vec![],
            warnings: vec![w],
        }
    }

    // Useful for recovering the type checker to continue typechecking
    // In other words, use this new type but keep the previous errors
    fn recover_to(self, a: A) -> Outcome<A> {
        Outcome {
            result: Some(a),
            errors: self.errors,
            warnings: self.warnings,
        }
    }

    #[allow(dead_code)]
    fn map<B>(self, f: impl FnOnce(A) -> B) -> Outcome<B> {
        match self.result {
            Some(t) => Outcome {
                result: Some(f(t)),
                errors: self.errors,
                warnings: self.warnings,
            },
            None => Outcome {
                result: None,
                errors: self.errors,
                warnings: self.warnings,
            },
        }
    }

    // Monadic bind, also known as `bind`, `andThen`, `then`, `let*` and `>>=`
    // In summary: take an `Outcome` then map over it and flatten
    fn and_then<B>(mut self, f: impl FnOnce(A) -> Outcome<B>) -> Outcome<B> {
        match self.result {
            Some(o) => {
                let mut outcome = f(o);
                self.errors.append(&mut outcome.errors);
                self.warnings.append(&mut outcome.warnings);

                Outcome {
                    result: outcome.result,
                    errors: self.errors,
                    warnings: self.warnings,
                }
            }
            None => Outcome {
                result: None,
                errors: self.errors,
                warnings: self.warnings,
            },
        }
    }

    // Monoidal product, also known as `and*`
    // In summary: take two `Outcome`s,
    // If they are both successful then return tuple of both values
    // Otherwise return `None`
    fn and_zip<B>(mut self, mut other: Outcome<B>) -> Outcome<(A, B)> {
        self.errors.append(&mut other.errors);
        self.warnings.append(&mut other.warnings);

        match (self.result, other.result) {
            (Some(l), Some(r)) => Outcome {
                result: Some((l, r)),
                errors: self.errors,
                warnings: self.warnings,
            },
            (_, _) => Outcome {
                result: None,
                errors: self.errors,
                warnings: self.warnings,
            },
        }
    }
}

fn infer(e: &ast::SpanExpr) -> Outcome<ast::JustType> {
    match e {
        ast::Expr::IntLit(_, _) => Outcome::new(ast::Type::Int(())),
        ast::Expr::BoolLit(_, _) => Outcome::new(ast::Type::Bool(())),
        ast::Expr::TypeAnno { term, ty, .. } => infer(term)
            .and_then(|term_ty| {
                // If inferring the type was successful, then check if the annotation matches the inferred type
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
            // Recover to the annotated type no matter what
            .recover_to(ty.strip()),
        ast::Expr::IfFlow {
            cond,
            on_true,
            on_false,
            extra,
        } => infer(cond)
            .and_then(|ty| {
                // If inferring the type was successful, then check that condition is of the type `bool`
                if ty.strip() == ast::Type::Bool(()) {
                    match **cond {
                        ast::Expr::BoolLit(span, b) => {
                            Outcome::new_warn(TypeWarning::CondAlways { span, value: b })
                        }
                        _ => Outcome::new_empty(),
                    }
                } else {
                    Outcome::new_err(TypeError::IfCondMustBeBool {
                        end: cond.extra().1,
                        got: ty,
                    })
                }
            })
            .recover_to(())
            // Check if both branches of the if are of the same type
            // `(_)` means we ignore whatever value is being passes, cause its `Unit` in this case
            .and_then(|_| infer(on_true).and_zip(infer(on_false)))
            // If they are of different types, don't recover cause it can't be known which one is the "correct" type
            .and_then(|(first, second)| {
                if first == second {
                    Outcome::new(first)
                } else {
                    Outcome::new_err(TypeError::IfBranchesMustBeSame {
                        start: extra.0,
                        first,
                        second,
                    })
                }
            }),
    }
}

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub result: Result<ast::JustType, Vec<TypeError>>,
    pub warnings: Vec<TypeWarning>,
}

// Wrapper for `infer`, so that it has a safer API
pub fn check(e: &ast::SpanExpr) -> CheckResult {
    let inferred = infer(e);

    // If there are any errors, then return only the errors
    if inferred.errors.len() > 0 {
        CheckResult {
            result: Err(inferred.errors),
            warnings: inferred.warnings,
        }
    } else {
        CheckResult {
            // This unwraps the `Some`, since we already verified it should have some value
            result: Ok(inferred.result.unwrap()),
            warnings: inferred.warnings,
        }
    }
}
