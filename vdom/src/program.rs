use crate::{cmd::Cmd, view::Memo, CachedView, ClosureId, Diff, Handler, Node, PatchNode};
use serde_derive::{Deserialize, Serialize};
use std::{
    any::Any,
    collections::HashMap,
    mem::replace,
    rc::{Rc, Weak},
    task::Context,
    unimplemented,
};

pub trait Program: 'static {
    type Model: PartialEq;
    type Msg;
    fn init() -> Self::Model {
        unimplemented!()
    }
    fn init_cmd() -> (Self::Model, Cmd<Self::Msg>) {
        (Self::init(), Cmd::none())
    }
    fn update(_model: &Self::Model, _msg: &Self::Msg) -> Self::Model {
        unimplemented!()
    }
    fn update_cmd(model: &Self::Model, msg: &Self::Msg) -> (Self::Model, Cmd<Self::Msg>) {
        (Self::update(model, msg), Cmd::none())
    }
    fn view(model: &Self::Model) -> Node<Self::Msg>;
    fn subscriptions() {}
}

pub struct Manager<P: Program> {
    view: CachedView<P::Msg>,
    model: Rc<P::Model>,
    cmd: Cmd<P::Msg>,
    handlers: HashMap<ClosureId, Weak<dyn Any>>,
}

impl<P: Program> Manager<P> {
    pub fn new() -> Self {
        let (model, cmd) = P::init_cmd();
        let model = Rc::new(model);
        let view = CachedView::new(None, Memo::new(P::view, model.clone()));
        Self {
            view,
            model,
            cmd,
            handlers: Default::default(),
        }
    }

    pub fn full_render(&mut self) -> &mut Node<P::Msg> {
        let node = self.view.full_render();
        node.pick_handler(&mut self.handlers);
        node
    }

    pub fn on_msg(&mut self, msg: &P::Msg) {
        let (model, cmd) = P::update_cmd(&self.model, msg);
        if !cmd.is_none() {
            if self.cmd.is_none() {
                self.cmd = cmd;
            } else {
                self.cmd = Cmd::batch(vec![replace(&mut self.cmd, Cmd::none()), cmd]);
            }
        }
        self.model = Rc::new(model);
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

    pub fn resolve(&mut self, context: &mut Context) {
        while let Some(msg) = self.cmd.try_get(context) {
            self.on_msg(&msg)
        }
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
