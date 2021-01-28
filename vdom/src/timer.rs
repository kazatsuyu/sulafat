use crate::cmd::Cmd;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

#[cfg(target_arch = "wasm32")]
use {
    crate::cmd::get_trriger,
    std::{cell::Cell, cell::RefCell, mem::replace, rc::Rc},
    wasm_bindgen::prelude::Closure,
    wasm_bindgen::JsCast,
    web_sys::window,
};

#[cfg(not(target_arch = "wasm32"))]
use chrono::{DateTime, Duration, Utc};

#[cfg(target_arch = "wasm32")]
pub fn timeout<Msg, F>(f: F, ms: i32) -> Cmd<Msg>
where
    F: 'static + Fn() -> Msg,
    Msg: 'static,
{
    struct Timeout<F, Msg>
    where
        F: 'static + Fn() -> Msg,
        Msg: 'static,
    {
        ready: Rc<Cell<bool>>,
        trigger: Option<Rc<RefCell<dyn FnMut()>>>,
        closure: Option<Closure<dyn FnMut()>>,
        ms: i32,
        f: F,
    }
    impl<F, Msg> Future for Timeout<F, Msg>
    where
        F: 'static + Fn() -> Msg,
        Msg: 'static,
    {
        type Output = Msg;
        fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
            let this = unsafe { Pin::get_unchecked_mut(self) };
            if this.ready.get() {
                Poll::Ready((this.f)())
            } else {
                if let Some(trigger) = replace(&mut this.trigger, None) {
                    let weak = Rc::downgrade(&this.ready);
                    let closure = Closure::once(Box::new(move || {
                        if let Some(timeout) = weak.upgrade() {
                            timeout.set(true);
                            trigger.borrow_mut()();
                        }
                    }));
                    window()
                        .unwrap()
                        .set_timeout_with_callback_and_timeout_and_arguments_0(
                            closure.as_ref().unchecked_ref(),
                            this.ms,
                        )
                        .unwrap();
                    this.closure = Some(closure);
                }
                Poll::Pending
            }
        }
    }
    Cmd::promise(Timeout {
        ready: Rc::new(Cell::new(false)),
        trigger: Some(get_trriger()),
        closure: None,
        ms,
        f,
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn timeout<Msg, F>(f: F, ms: i32) -> Cmd<Msg>
where
    F: 'static + Fn() -> Msg,
    Msg: 'static,
{
    struct Timeout(DateTime<Utc>);
    impl Future for Timeout {
        type Output = ();
        fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
            if Utc::now() > self.0 {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        }
    }

    Cmd::promise(Timeout(Utc::now() + Duration::microseconds(ms as i64))).map(move |()| f())
}
