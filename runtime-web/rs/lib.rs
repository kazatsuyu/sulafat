mod utils;

use bincode::{deserialize, serialize};
use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    thread_local,
};
use sulafat_macros::StyleSet;
use sulafat_vdom::{
    cmd::Cmd, on_click, random::range, style, timer::timeout, Common, Div, EventHandler, Manager,
    Node, Program,
};
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

enum Msg {
    Update,
    Timeout,
    Random(u32),
}

#[derive(PartialEq)]
struct Model {
    len: usize,
    rand: u32,
    count: u32,
}

impl Program for MyProgram {
    type Model = Model;
    type Msg = Msg;
    fn init_cmd() -> (Self::Model, Cmd<Self::Msg>) {
        (
            Model {
                len: 0,
                rand: 0,
                count: 0,
            },
            Cmd::batch(vec![
                timeout(|| Msg::Timeout, 1000),
                range(Msg::Random, 0..10),
            ]),
        )
    }
    fn update_cmd(model: &Self::Model, msg: &Self::Msg) -> (Self::Model, Cmd<Self::Msg>) {
        match *msg {
            Msg::Update => (
                Model {
                    len: model.len + 1,
                    ..*model
                },
                range(Msg::Random, 0..10),
            ),
            Msg::Timeout => (
                Model {
                    count: model.count + 1,
                    ..*model
                },
                timeout(|| Msg::Timeout, 1000),
            ),
            Msg::Random(rand) => (Model { rand, ..*model }, Cmd::none()),
        }
    }
    fn view(model: &Self::Model) -> Node<Self::Msg> {
        let children = (0..model.len + 2).map(|index| {
            let text = match index {
                0 => "Update".into(),
                1 => format!("{}, {}, {}", model.len, model.rand, model.count),
                _ => format!("{}", index - 2),
            };

            #[derive(StyleSet)]
            #[style_set{
                .style1 {
                    left: 10px;
                }
            }]
            struct Style1;

            #[derive(StyleSet)]
            #[style_set{
                .style2 {
                    left: 20px;
                }
            }]
            struct Style2;

            let attr = if index == 0 {
                vec![on_click(|_| Msg::Update)]
            } else {
                vec![if index % 2 == 1 {
                    style(Style1)
                } else {
                    style(Style2)
                }]
            };
            Div::new(Common::new(None, attr.into(), vec![text.into()].into())).into()
        });
        children.collect()
    }
}

thread_local! {
    static MANAGER: RefCell<Box<Manager<MyProgram>>> = RefCell::new(Manager::new());
}

#[wasm_bindgen]
pub fn internal_init() -> Vec<u8> {
    utils::set_panic_hook();
    wasm_bindgen_futures::spawn_local(Resolver);
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

struct Resolver;

impl Future for Resolver {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        MANAGER.with(|manager| {
            manager.borrow_mut().resolve(cx);
        });
        Poll::Ready(())
    }
}

#[wasm_bindgen]
pub fn internal_on_event(data: Vec<u8>) {
    MANAGER.with(|manager| {
        let mut manager = manager.borrow_mut();
        manager.on_event(&deserialize::<EventHandler>(&data).unwrap());
        wasm_bindgen_futures::spawn_local(Resolver)
    })
}
