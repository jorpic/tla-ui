mod utils;

use wee_alloc::WeeAlloc;

#[global_allocator]
static ALLOC: WeeAlloc = WeeAlloc::INIT;

use wasm_bindgen::prelude::*;

fn car_cdr(s: &str) -> (&str, &str) {
    match s.chars().next() {
        Some(c) => s.split_at(c.len_utf8()),
        None => s.split_at(0),
    }
}

#[wasm_bindgen]
pub fn parse(tla: &str) -> Option<String> {
    let (first_char, _remainder) = car_cdr(tla);
    if first_char == "#" {
        Some("comment".into())
    } else if first_char == "$" {
        Some("keyword".into())
    } else {
        None
    }
}
