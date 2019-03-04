use pest::{Parser, Token};
use tla_parser::{Rule, TlaParser};

#[test]
fn pass() {
    let text = "---- MODULE xxx ----
    EXTENDS Integers, TLC, Sequences
    ====";

    let res = TlaParser::parse(Rule::tla_module, text)
        .unwrap_or_else(|err| panic!("{}", err));
    for pair in res.tokens() {
        match pair {
            Token::Start{rule, pos} => println!("Start {:?} {:?}", rule, pos.line_col()),
            Token::End{rule, pos}   => println!("End   {:?} {:?}", rule, pos.line_col()),
        }
    }
}
