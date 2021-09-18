use crate::ast;

// A customized result type
#[derive(Clone, Debug)]
struct Outcome<A> {
    ok: Option<A>,
    errors: Vec<String>,
    warnings: Vec<String>,
}

impl<A> Outcome<A> {
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

    fn with_warn(mut self, s: String) -> Outcome<A> {
        self.warnings.push(s);
        self
    }

    fn map<B>(self, f: impl FnOnce(A) -> B) -> Outcome<B> {
        match self.ok {
            Some(t) => Outcome { ok: Some(f(t)), errors: self.errors, warnings: self.warnings },
            None => Outcome { ok: None, errors: self.errors, warnings: self.warnings},
        }
    }

    fn and_then<B>(mut self, f: impl FnOnce(A) -> Outcome<B>) -> Outcome<B> {
        match self.ok {
            Some(o) => {
                let mut outcome = f(o);
                self.errors.append(&mut outcome.errors);
                self.warnings.append(&mut outcome.warnings);

                match outcome.ok {
                    Some(o) => Outcome { ok: Some(o), errors: self.errors, warnings: self.warnings },
                    None => Outcome { ok: None, errors: self.errors, warnings: self.warnings },
                }
            },
            None => Outcome { ok: None, errors: self.errors, warnings: self.warnings },
        }
    }

    fn and_zip<B>(mut self, mut other: Outcome<B>) -> Outcome<(A, B)> {
        self.errors.append(&mut other.errors);
        self.warnings.append(&mut other.warnings);

        match (self.ok, other.ok) {
            (Some(l), Some(r)) => Outcome { ok: Some((l, r)), errors: self.errors, warnings: self.warnings },
            (None, _) => Outcome { ok: None, errors: self.errors, warnings: self.warnings },
            (_, None) => Outcome { ok: None, errors: self.errors, warnings: self.warnings },
        }
    }
}