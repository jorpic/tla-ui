#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;
use tla_parser::parse;

#[wasm_bindgen_test]
fn pass() {
    assert_eq!(parse("# hello"), Some("comment".into()));
    assert_eq!(parse("! hello"), None);
}
