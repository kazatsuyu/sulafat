#![cfg_attr(feature = "nightly-features", feature(hash_raw_entry))]

mod utils;

use std::sync::Mutex;

use bincode::serialize;
use once_cell::sync::Lazy;
use sulafat_vdom::{Common, Diff, Div, List, Node};
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static NODE: Lazy<Mutex<Node>> = Lazy::new(|| Mutex::new(List::default().into()));
static COUNT: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(0));

#[wasm_bindgen]
pub fn internal_init() -> Vec<u8> {
    utils::set_panic_hook();
    serialize(&*NODE.lock().unwrap()).unwrap()
}

#[wasm_bindgen]
pub fn internal_update() -> Option<Vec<u8>> {
    let mut count = COUNT.lock().unwrap();
    let mut prev = NODE.lock().unwrap();
    *count += 1;
    let children = (0..=*count).map(|index| {
        Div::new(Common::new(
            None,
            None,
            vec![format!("{}", if index == 0 { *count } else { index - 1 }).into()].into(),
        ))
        .into()
    });
    let node: Node = children.collect();
    let patch = prev.diff(&node);
    *prev = node;
    patch.map(|patch| serialize(&patch).unwrap())
}
