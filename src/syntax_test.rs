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

        let max = &u64::MAX.to_string();
        assert!(TermParser::new().parse(max).is_ok());
        assert!(TermParser::new().parse("-1").is_err());
        assert!(TermParser::new().parse(&((u64::MAX as u128) + 1).to_string()).is_err());
    }

    #[test]
    fn anno() {
        assert!(TermParser::new().parse("1: nat").is_ok());
        assert!(TermParser::new().parse("nat: 1").is_err());
    }

    #[test]
    fn if_flow() {
        assert!(TermParser::new().parse("if true then 1 else 0 end").is_ok());
    }

    #[test]
    fn scratch() {

    }
}