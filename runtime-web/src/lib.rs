#![cfg_attr(feature = "nightly-features", feature(hash_raw_entry))]

mod utils;

use serde_cbor::to_vec;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn internal_init() -> Vec<u8> {
    to_vec(&vdom::List::default()).unwrap()
}
