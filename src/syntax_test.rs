// TODO: add error recovery?

#[cfg(test)]
mod tests {
    use crate::syntax::TermParser;

    #[test]
    fn bool_lit() {
        assert!(TermParser::new().parse("true").is_ok());
        assert!(TermParser::new().parse("false").is_ok());
    }

    #[test]
    fn nat_lit() {
        assert!(TermParser::new().parse("0").is_ok());

        let max = &i64::MAX.to_string();
        let min = &i64::MIN.to_string();
        assert!(TermParser::new().parse(max).is_ok());
        assert!(TermParser::new().parse(min).is_ok());

        assert!(TermParser::new().parse(&((i64::MAX as i128) + 1).to_string()).is_err());
        assert!(TermParser::new().parse(&((i64::MIN as i128) - 1).to_string()).is_err());
    }

    #[test]
    fn anno() {
        assert!(TermParser::new().parse("1: int").is_ok());
        assert!(TermParser::new().parse("int: 1").is_err());
    }

    #[test]
    fn if_flow() {
        assert!(TermParser::new().parse("if true then 1 else 0 end").is_ok());
    }

    #[test]
    fn types() {
        assert!(TermParser::new().parse("true").is_ok());
        assert!(TermParser::new().parse("1").is_ok());
    }
}