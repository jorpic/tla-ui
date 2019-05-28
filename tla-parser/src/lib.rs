use wee_alloc::WeeAlloc;

#[global_allocator]
static ALLOC: WeeAlloc = WeeAlloc::INIT;

use wasm_bindgen::prelude::*;
mod lexer;


#[wasm_bindgen]
pub struct ParseTree;


#[wasm_bindgen]
impl ParseTree {
    pub fn get_style(&self, line: u32, _column: u32) -> Option<String> {
        if line == 2 {
            Some("keyword".into())
        } else {
            Some("comment".into())
        }
    }
}


#[wasm_bindgen]
pub fn parse(_code: &str) -> Result<ParseTree, JsValue> {
    Ok(ParseTree{})
}
