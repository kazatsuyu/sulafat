use crate::{view::Memo, CachedView, ClosureId, Diff, Handler, Node, PatchNode};
use serde_derive::{Deserialize, Serialize};
use std::{
    any::Any,
    collections::HashMap,
    rc::{Rc, Weak},
};

pub trait Program: 'static {
    type Model: PartialEq;
    type Msg;
    fn init() -> Self::Model;
    fn update(model: &Self::Model, msg: &Self::Msg) -> Self::Model;
    fn view(model: &Self::Model) -> Node<Self::Msg>;
}

pub struct Manager<P: Program> {
    view: CachedView<P::Msg>,
    model: Rc<P::Model>,
    handlers: HashMap<ClosureId, Weak<dyn Any>>,
}

impl<P: Program> Manager<P> {
    pub fn new() -> Self {
        let model = Rc::new(P::init());
        let view = CachedView::new(None, Memo::new(P::view, model.clone()));
        Self {
            view,
            model,
            handlers: Default::default(),
        }
    }

    pub fn full_render(&mut self) -> &mut Node<P::Msg> {
        let node = self.view.full_render();
        node.pick_handler(&mut self.handlers);
        node
    }

    pub fn on_msg(&mut self, msg: &P::Msg) {
        self.model = Rc::new(P::update(&self.model, msg));
    }

    pub fn on_event(&mut self, event_handler: &EventHandler) {
        let handler = self
            .handlers
            .get(&event_handler.closure_id)
            .unwrap()
            .upgrade()
            .unwrap();
        let msg = match &event_handler.event {
            Event::OnClick => handler
                .downcast::<Handler<(), P::Msg>>()
                .unwrap()
                .invoke(()),
            &Event::OnPointerMove(x, y) => handler
                .downcast::<Handler<(f64, f64), P::Msg>>()
                .unwrap()
                .invoke((x, y)),
        };
        self.on_msg(&msg)
    }

    pub fn diff(&mut self) -> Option<PatchNode> {
        let mut view = CachedView::new(None, Memo::new(P::view, self.model.clone()));
        let diff = self.view.diff(&mut view);
        self.handlers.clear();
        unsafe { view.rendered() }
            .unwrap()
            .pick_handler(&mut self.handlers);
        self.view = view;
        diff
    }
}

#[derive(Serialize, Deserialize)]
pub enum Event {
    OnClick,
    OnPointerMove(f64, f64),
}

#[derive(Serialize, Deserialize)]
pub struct EventHandler {
    closure_id: ClosureId,
    event: Event,
}
