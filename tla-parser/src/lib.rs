
use wee_alloc::WeeAlloc;

#[global_allocator]
static ALLOC: WeeAlloc = WeeAlloc::INIT;

mod utils;

#[macro_use]
extern crate pest_derive;
mod parser;
use pest::{Parser, Token};
pub use self::parser::{Rule, TlaParser};


use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct ParseTree;

#[wasm_bindgen]
impl ParseTree {
    fn from_tokens<'a, I>(tokens: I) -> Self
        where I : IntoIterator<Item=Token<'a, Rule>>
    {
        let tokens_iter = tokens.into_iter();
        ParseTree{}
    }

    // FIXME: stub code
    pub fn get_style(&self, line: u32, column: u32) -> Option<String> {
        if line == 2 {
            Some("keyword".into())
        } else {
            Some("comment".into())
        }
    }
}

#[wasm_bindgen]
pub fn parse(code: &str) -> Result<ParseTree, JsValue> {
    let res = TlaParser::parse(Rule::module, code)
        .map_err(|err| JsValue::from_str(&format!("{:?}", err)))?;

    Ok(ParseTree::from_tokens(res.tokens()))
}
