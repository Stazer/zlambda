use console_error_panic_hook::set_once;

pub fn set_panic_hook() {
    set_once();
}

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn greet() -> i64 {
    0xdeadbeef
}
