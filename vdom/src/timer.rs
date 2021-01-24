use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
#[cfg(target_arch = "wasm32")]
use {
    std::{cell::Cell, rc::Rc},
    wasm_bindgen::prelude::Closure,
    wasm_bindgen::JsCast,
    web_sys::window,
};

#[cfg(not(target_arch = "wasm32"))]
use chrono::{DateTime, Duration, Utc};

use crate::cmd::Cmd;

#[cfg(target_arch = "wasm32")]
pub fn timeout<Msg, F>(f: F, ms: i32) -> Cmd<Msg>
where
    F: 'static + Fn() -> Msg,
    Msg: 'static,
{
    struct Timeout(Rc<Cell<bool>>);
    impl Future for Timeout {
        type Output = ();
        fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
            if self.0.get() {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        }
    }
    let timeout = Timeout(Rc::new(Cell::new(false)));
    let weak = Rc::downgrade(&timeout.0);
    let closure = Closure::<dyn FnOnce()>::once(Box::new(move || {
        if let Some(timeout) = weak.upgrade() {
            timeout.set(true);
        }
    }));
    window()
        .unwrap()
        .set_timeout_with_callback_and_timeout_and_arguments_0(closure.as_ref().unchecked_ref(), ms)
        .unwrap();

    Cmd::promise(timeout).map(move |()| {
        &closure;
        f()
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
