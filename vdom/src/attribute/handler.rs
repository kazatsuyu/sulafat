use std::{
    any::Any,
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
    rc::Rc,
};

use crate::{util::force_cast, ClosureId};
use sulafat_macros::Serialize;

#[derive(Serialize)]
pub struct Handler<Args, Msg> {
    pub(crate) closure_id: ClosureId,
    #[serde(skip)]
    handle: Box<dyn Handle<Args, Msg>>,
}

impl<Args, Msg> Handler<Args, Msg> {
    pub fn invoke(&self, args: Args) -> Msg {
        self.handle.invoke(args)
    }

    pub(crate) fn new<F: 'static + Fn(Args) -> Msg>(f: F) -> Self
    where
        Args: 'static,
        Msg: 'static,
    {
        Self {
            closure_id: ClosureId::new::<F, Args, Msg>(&f),
            handle: Box::new(f),
        }
    }

    pub fn with_data<F: 'static + Fn(&Data, Args) -> Msg, Data>(f: F, data: Data) -> Self
    where
        Args: 'static,
        Msg: 'static,
        Data: 'static + PartialEq,
    {
        Self {
            closure_id: ClosureId::with_data::<F, &Data, Args, Msg>(&f),
            handle: Box::new(CachedHandle {
                data,
                f,
                _p: Default::default(),
            }),
        }
    }

    pub fn wrap<Msg2>(self) -> Handler<Args, Msg2>
    where
        Msg: 'static,
        Args: 'static,
        Msg2: 'static + From<Msg>,
    {
        force_cast(self).unwrap_or_else(|this| Handler {
            closure_id: this.closure_id,
            handle: Box::new(move |args| this.handle.invoke(args).into()),
        })
    }

    pub(crate) fn as_any(self: Rc<Self>) -> Rc<dyn Any>
    where
        Msg: 'static,
        Args: 'static,
    {
        self as Rc<dyn Any>
    }

    pub fn closure_id(&self) -> &ClosureId {
        &self.closure_id
    }
}

impl<Args, Msg> Debug for Handler<Args, Msg> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Handler({:?})", self.closure_id)
    }
}

impl<Args, Msg> PartialEq for Handler<Args, Msg> {
    fn eq(&self, other: &Self) -> bool {
        self.closure_id == other.closure_id && self.handle.is_same(&*other.handle)
    }
}

impl<Args, Msg> Eq for Handler<Args, Msg> {}

pub trait Handle<Args, Msg> {
    fn invoke(&self, args: Args) -> Msg;
    fn is_same(&self, other: &dyn Handle<Args, Msg>) -> bool;
    fn as_any(&self) -> &dyn Any;
}

impl<Msg, F, Args> Handle<Args, Msg> for F
where
    F: 'static + Fn(Args) -> Msg,
{
    fn invoke(&self, args: Args) -> Msg {
        self(args)
    }

    fn is_same(&self, other: &dyn Handle<Args, Msg>) -> bool {
        other.as_any().downcast_ref::<Self>().is_some()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct CachedHandle<F, Data, Args, Msg> {
    data: Data,
    f: F,
    _p: PhantomData<(Args, Msg)>,
}

impl<F, Data, Args, Msg> Handle<Args, Msg> for CachedHandle<F, Data, Args, Msg>
where
    F: Fn(&Data, Args) -> Msg,
    Self: 'static,
    Data: PartialEq,
{
    fn invoke(&self, args: Args) -> Msg {
        (self.f)(&self.data, args)
    }

    fn is_same(&self, other: &dyn Handle<Args, Msg>) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.data == other.data
        } else {
            false
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
