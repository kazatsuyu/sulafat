mod utils;

use bincode::serialize;
use std::{cell::RefCell, thread_local};
use sulafat_vdom::{Common, Diff, Div, List, Node};
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

thread_local! {
    static NODE: RefCell<Node<()>> = RefCell::new(List::default().into());
    static COUNT: RefCell<usize> = RefCell::new(0);
}

#[wasm_bindgen]
pub fn internal_init() -> Vec<u8> {
    utils::set_panic_hook();
    NODE.with(|node| serialize(&*node.borrow_mut())).unwrap()
}

#[wasm_bindgen]
pub fn internal_update() -> Option<Vec<u8>> {
    COUNT.with(|count| {
        NODE.with(|prev| {
            let count = &mut *count.borrow_mut();
            let prev = &mut *prev.borrow_mut();
            *count += 1;
            let children = (0..=*count).map(|index| {
                Div::new(Common::new(
                    None,
                    None,
                    vec![format!("{}", if index == 0 { *count } else { index - 1 }).into()].into(),
                ))
                .into()
            });
            let node: Node<()> = children.collect();
            let patch = prev.diff(&node);
            *prev = node;
            patch.map(|patch| serialize(&patch).unwrap())
        })
    })
}
