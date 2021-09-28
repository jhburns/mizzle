use crate::ast;
use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub enum TypeError {
    AnnotationIncorrect { span: ast::Span, got: ast::JustType, annotation: ast::JustType },
    IfCondMustBeBool { end: usize, got: ast::JustType },
    IfBranchesMustBeSame { start: usize, first: ast::JustType, second: ast::JustType },
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
    CondAlways { span: ast::Span, value: bool }
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

// A customized result type
#[must_use]
#[derive(Clone, Debug)]
struct Outcome<A> {
    pub result: Option<A>,
    pub errors: Vec<TypeError>,
    pub warnings: Vec<TypeWarning>,
}

impl<A> Outcome<A> {
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

    fn set_result(self, a: A) -> Outcome<A> {
        Outcome { result: Some(a), errors: self.errors, warnings: self.warnings }
    }

    #[allow(dead_code)]
    fn map<B>(self, f: impl FnOnce(A) -> B) -> Outcome<B> {
        match self.result {
            Some(t) => Outcome { result: Some(f(t)), errors: self.errors, warnings: self.warnings },
            None => Outcome { result: None, errors: self.errors, warnings: self.warnings },
        }
    }

    fn and_then<B>(mut self, f: impl FnOnce(A) -> Outcome<B>) -> Outcome<B> {
        match self.result {
            Some(o) => {
                let mut outcome = f(o);
                self.errors.append(&mut outcome.errors);
                self.warnings.append(&mut outcome.warnings);

                Outcome { result: outcome.result, errors: self.errors, warnings: self.warnings }
            },
            None => Outcome { result: None, errors: self.errors, warnings: self.warnings },
        }
    }

    fn and_zip<B>(mut self, mut other: Outcome<B>) -> Outcome<(A, B)> {
        self.errors.append(&mut other.errors);
        self.warnings.append(&mut other.warnings);

        match (self.result, other.result) {
            (Some(l), Some(r)) => Outcome { result: Some((l, r)), errors: self.errors, warnings: self.warnings },
            (_, _) => Outcome { result: None, errors: self.errors, warnings: self.warnings },
        }
    }
}

fn infer(e: &ast::SpanExpr) -> Outcome<ast::JustType> {
    match e {
        ast::Expr::IntLit(_, _) => Outcome::new(ast::Type::Int(())),
        ast::Expr::BoolLit(_, _) => Outcome::new(ast::Type::Bool(())),
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
                .set_result(ty.strip())
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
                .set_result(ast::Type::Bool(()))
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

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub result: Result<ast::JustType, Vec<TypeError>>,
    pub warnings: Vec<TypeWarning>,
}

pub fn check(e: &ast::SpanExpr) -> CheckResult {
    let inferred = infer(e);

    if inferred.errors.len() > 0 {
        CheckResult { result: Err(inferred.errors), warnings: inferred.warnings }
    } else {
        CheckResult { result: Ok(inferred.result.unwrap()), warnings: inferred.warnings }
    }
}