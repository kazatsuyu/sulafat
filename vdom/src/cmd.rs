use std::{
    future::Future,
    marker::PhantomData,
    mem::replace,
    pin::Pin,
    task::{Context, Poll},
};

enum CmdInner<Msg> {
    Fn(Box<dyn FnOnce() -> Msg>),
    Future(Pin<Box<dyn Future<Output = Msg>>>),
    List(Vec<Cmd<Msg>>),
    Map(Box<dyn Map<Msg>>),
}

trait Map<Msg> {
    fn is_none(&self) -> bool;
    fn try_get(&mut self, context: &mut Context) -> Option<Msg>;
}

struct MapImpl<Msg, Msg2, F> {
    cmd: Cmd<Msg>,
    f: F,
    _p: PhantomData<Msg2>,
}

impl<Msg, Msg2, F> Map<Msg2> for MapImpl<Msg, Msg2, F>
where
    F: 'static + Fn(Msg) -> Msg2,
{
    fn is_none(&self) -> bool {
        self.cmd.is_none()
    }
    fn try_get(&mut self, context: &mut Context) -> Option<Msg2> {
        Some((self.f)(self.cmd.try_get(context)?))
    }
}

pub struct Cmd<Msg>(CmdInner<Msg>);

impl<Msg> Cmd<Msg> {
    pub fn none() -> Self {
        Self(CmdInner::List(vec![]))
    }

    pub fn batch(mut list: Vec<Cmd<Msg>>) -> Self {
        let mut i = 0;
        while i < list.len() {
            if list[i].is_none() {
                list.swap_remove(i);
            } else {
                i += 1;
            }
        }
        if list.len() == 1 {
            return list.pop().unwrap();
        }
        Self(CmdInner::List(list))
    }

    pub fn map<F, Msg2>(self, f: F) -> Cmd<Msg2>
    where
        F: 'static + Fn(Msg) -> Msg2,
        Msg2: 'static,
        Msg: 'static,
    {
        if self.is_none() {
            return Cmd::none();
        }
        Cmd(CmdInner::Map(Box::new(MapImpl {
            cmd: self,
            f,
            _p: PhantomData,
        })))
    }

    pub(crate) fn with<F>(f: F) -> Self
    where
        F: 'static + FnOnce() -> Msg,
    {
        Self(CmdInner::Fn(Box::new(f)))
    }

    pub(crate) fn promise<F>(f: F) -> Self
    where
        F: 'static + Future<Output = Msg>,
    {
        Self(CmdInner::Future(Box::pin(f)))
    }

    pub(crate) fn is_none(&self) -> bool {
        match &self.0 {
            CmdInner::List(list) => list.is_empty(),
            CmdInner::Map(map) => map.is_none(),
            CmdInner::Fn(_) | CmdInner::Future(_) => false,
        }
    }

    pub(crate) fn try_get(&mut self, context: &mut Context) -> Option<Msg> {
        match replace(self, Self::none()).0 {
            CmdInner::Fn(f) => Some(f()),
            CmdInner::Future(mut future) => match future.as_mut().poll(context) {
                Poll::Pending => {
                    *self = Self(CmdInner::Future(future));
                    None
                }
                Poll::Ready(msg) => Some(msg),
            },
            CmdInner::List(mut list) => {
                for i in 0..list.len() {
                    if let Some(msg) = list[i].try_get(context) {
                        list.swap_remove(i);
                        if !list.is_empty() {
                            *self = Self(CmdInner::List(list));
                        }
                        return Some(msg);
                    }
                }
                if !list.is_empty() {
                    *self = Self(CmdInner::List(list));
                }
                None
            }
            CmdInner::Map(mut map) => {
                if let Some(msg) = map.try_get(context) {
                    if !map.is_none() {
                        *self = Self(CmdInner::Map(map));
                    }
                    Some(msg)
                } else {
                    *self = Self(CmdInner::Map(map));
                    None
                }
            }
        }
    }
}
