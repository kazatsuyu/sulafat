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

#[cfg(target_arch = "wasm32")]
use {
    crate::cmd::{register_trigger, unregister_trigger},
    std::{
        future::Future,
        mem::{forget, MaybeUninit},
        pin::Pin,
        ptr::null_mut,
        task::Poll,
    },
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
    #[cfg(target_arch = "wasm32")]
    weak: WeakManager<P>,
}

impl<P: Program> Manager<P> {
    #[cfg(target_arch = "wasm32")]
    pub fn new() -> Box<Self> {
        let mut uninit_this = Box::new(MaybeUninit::uninit());
        let ptr = uninit_this.as_mut_ptr();
        let mut weak = WeakManager::new();
        weak.set(ptr);
        register_trigger(weak.clone());
        let (model, cmd) = P::init_cmd();
        unregister_trigger();
        let model = Rc::new(model);
        let view = CachedView::new(None, Memo::new(P::view, model.clone()));
        let this = Self {
            view,
            model,
            cmd,
            handlers: Default::default(),
            weak,
        };
        unsafe { ptr.write(this) };
        forget(uninit_this);
        unsafe { Box::from_raw(ptr) }
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new() -> Box<Self> {
        let (model, cmd) = P::init_cmd();
        let model = Rc::new(model);
        let view = CachedView::new(None, Memo::new(P::view, model.clone()));
        Box::new(Self {
            view,
            model,
            cmd,
            handlers: Default::default(),
        })
    }

    pub fn full_render(&mut self) -> &mut Node<P::Msg> {
        let node = self.view.full_render();
        node.pick_handler(&mut self.handlers);
        node
    }

    pub fn on_msg(&mut self, msg: &P::Msg) {
        #[cfg(target_arch = "wasm32")]
        register_trigger(self.weak.clone());
        let (model, cmd) = P::update_cmd(&self.model, msg);
        #[cfg(target_arch = "wasm32")]
        unregister_trigger();
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

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn trigger(&mut self) {
        struct Trigger<P: Program>(WeakManager<P>);
        impl<P: Program> Future for Trigger<P> {
            type Output = ();
            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                if let Some(manager) = unsafe { self.0.get_mut() } {
                    manager.resolve(cx);
                }
                Poll::Ready(())
            }
        }
        {
            wasm_bindgen_futures::spawn_local(Trigger(self.weak.clone()))
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl<P: Program> Drop for Manager<P> {
    fn drop(&mut self) {
        self.weak.reset()
    }
}

#[cfg(target_arch = "wasm32")]
pub struct WeakManager<P: Program> {
    inner: *mut WeakManagerInner<P>,
}

#[cfg(target_arch = "wasm32")]
struct WeakManagerInner<P: Program> {
    count: u64,
    ptr: *mut Manager<P>,
}

#[cfg(target_arch = "wasm32")]
impl<P: Program> WeakManager<P> {
    fn new() -> Self {
        Self {
            inner: Box::leak(Box::new(WeakManagerInner {
                count: 1,
                ptr: null_mut(),
            })),
        }
    }
    fn set(&mut self, mgr: *mut Manager<P>) {
        unsafe { &mut *self.inner }.ptr = mgr;
    }
    fn reset(&mut self) {
        unsafe { &mut *self.inner }.ptr = null_mut();
    }
    pub(crate) unsafe fn get_mut(&mut self) -> Option<&mut Manager<P>> {
        let ptr = unsafe { &mut *self.inner }.ptr;
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { &mut *ptr })
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl<P: Program> Drop for WeakManager<P> {
    fn drop(&mut self) {
        let is_zero = {
            let count = &mut unsafe { &mut *self.inner }.count;
            *count -= 1;
            *count == 0
        };
        if is_zero {
            let ptr = unsafe { &*self.inner }.ptr;
            unsafe { Box::from_raw(ptr) };
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl<P: Program> Clone for WeakManager<P> {
    fn clone(&self) -> Self {
        unsafe { &mut *self.inner }.count += 1;
        Self { inner: self.inner }
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
