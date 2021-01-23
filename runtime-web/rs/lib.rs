mod utils;

use bincode::{deserialize, serialize};
use std::{cell::RefCell, thread_local};
use sulafat_macros::StyleSet;
use sulafat_vdom::{on_click, Common, Div, EventHandler, Manager, Node, Program};
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

struct MyProgram;

impl Program for MyProgram {
    type Model = usize;
    type Msg = ();
    fn init() -> Self::Model {
        0
    }
    fn update(model: &Self::Model, _msg: &Self::Msg) -> Self::Model {
        model + 1
    }
    fn view(model: &Self::Model) -> Node<Self::Msg> {
        let children = (0..*model + 2).map(|index| {
            let text = match index {
                0 => "Update".into(),
                1 => format!("{}", *model),
                _ => format!("{}", index - 2),
            };
            let attr = if index == 0 {
                vec![on_click(|_| ())]
            } else {
                vec![]
            };
            Div::new(Common::new(None, attr.into(), vec![text.into()].into())).into()
        });
        children.collect()
    }
}

thread_local! {
    static MANAGER: RefCell<Manager<MyProgram>> = RefCell::new(Manager::new());
}

#[wasm_bindgen]
pub fn internal_init() -> Vec<u8> {
    utils::set_panic_hook();
    MANAGER
        .with(|manager| serialize(manager.borrow_mut().full_render()))
        .unwrap()
}

#[wasm_bindgen]
pub fn internal_render() -> Option<Vec<u8>> {
    MANAGER.with(|manager| {
        let mut manager = manager.borrow_mut();
        manager.diff().map(|diff| serialize(&diff).unwrap())
    })
}

#[wasm_bindgen]
pub fn internal_on_event(data: Vec<u8>) {
    MANAGER.with(|manager| {
        let mut manager = manager.borrow_mut();
        manager.on_event(&deserialize::<EventHandler>(&data).unwrap());
    })
}

#[derive(StyleSet)]
#[style_set{
    left: 100px;
    right: 80%;
}]
struct Style;

#[derive(StyleSet)]
#[style_set{
    .style2 {
        left: 100px;
        right: 80%;
    }
}]
struct Style2;
