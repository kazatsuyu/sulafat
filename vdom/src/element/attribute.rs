use crate::{util::force_cast, ClosureId};
use serde::{ser::SerializeTupleVariant, Serialize, Serializer};
use std::{
    any::Any,
    collections::HashMap,
    fmt::Debug,
    fmt::{self, Formatter},
    hash::Hash,
    marker::PhantomData,
    rc::{Rc, Weak},
};
use sulafat_macros::{Serialize, VariantIdent};

#[derive(Serialize)]
pub struct Handler<Args, Msg> {
    closure_id: ClosureId,
    #[serde(skip)]
    handle: Box<dyn Handle<Args, Msg>>,
}

impl<Args, Msg> Handler<Args, Msg> {
    pub fn invoke(&self, args: Args) -> Msg {
        self.handle.invoke(args)
    }

    fn new<F: 'static + Fn(Args) -> Msg>(f: F) -> Self
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

    fn as_any(self: Rc<Self>) -> Rc<dyn Any>
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

#[derive(Debug, VariantIdent)]
pub enum Attribute<Msg> {
    Id(String),
    OnClick(Rc<Handler<(), Msg>>),
    OnPointerMove(Rc<Handler<(f64, f64), Msg>>),
}

impl<Msg> Attribute<Msg> {
    pub(crate) fn pick_handler(&self, handlers: &mut HashMap<ClosureId, Weak<dyn Any>>)
    where
        Msg: 'static,
    {
        match self {
            Attribute::Id(_) => {}
            Attribute::OnClick(handler) => {
                handlers.insert(handler.closure_id, Rc::downgrade(&handler.clone().as_any()));
            }
            Attribute::OnPointerMove(handler) => {
                handlers.insert(handler.closure_id, Rc::downgrade(&handler.clone().as_any()));
            }
        }
    }
}

pub fn id<Msg>(s: String) -> Attribute<Msg> {
    Attribute::Id(s)
}

pub fn on_click<Msg, F>(f: F) -> Attribute<Msg>
where
    F: 'static + Fn(()) -> Msg,
    Msg: 'static,
{
    Attribute::OnClick(Rc::new(Handler::new(f)))
}

pub fn on_pointer_move<Msg, F>(f: F) -> Attribute<Msg>
where
    F: 'static + Fn((f64, f64)) -> Msg,
    Msg: 'static,
{
    Attribute::OnPointerMove(Rc::new(Handler::new(f)))
}

impl<Args, Msg> PartialEq for Handler<Args, Msg> {
    fn eq(&self, other: &Self) -> bool {
        self.closure_id == other.closure_id && self.handle.is_same(&*other.handle)
    }
}

impl<Args, Msg> Eq for Handler<Args, Msg> {}

impl<Msg> PartialEq for Attribute<Msg> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Attribute::Id(this), Attribute::Id(other)) => this == other,
            (Attribute::OnClick(this), Attribute::OnClick(other)) => this == other,
            (Attribute::OnPointerMove(this), Attribute::OnPointerMove(other)) => this == other,
            _ => false,
        }
    }
}

impl<Msg> Eq for Attribute<Msg> {}

impl<Msg> Clone for Attribute<Msg> {
    fn clone(&self) -> Self {
        match self {
            Attribute::Id(id) => Attribute::Id(id.clone()),
            Attribute::OnClick(handler) => Attribute::OnClick(handler.clone()),
            Attribute::OnPointerMove(handler) => Attribute::OnPointerMove(handler.clone()),
        }
    }
}

impl<Msg> Serialize for Attribute<Msg> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Attribute::Id(id) => {
                let mut variant = serializer.serialize_tuple_variant("Attribute", 0, "Id", 1)?;
                variant.serialize_field(id)?;
                variant.end()
            }
            Attribute::OnClick(handler) => {
                let mut variant =
                    serializer.serialize_tuple_variant("Attribute", 1, "OnClick", 1)?;
                variant.serialize_field(handler.as_ref())?;
                variant.end()
            }
            Attribute::OnPointerMove(handler) => {
                let mut variant =
                    serializer.serialize_tuple_variant("Attribute", 2, "OnPointerMove", 1)?;
                variant.serialize_field(handler.as_ref())?;
                variant.end()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::Handler;

    #[test]
    fn handler_ne() {
        let h1 = Handler::new(|()| ());
        let h2 = Handler::new(|()| ());
        assert_ne!(h1, h2);
    }

    #[test]
    fn function_ne() {
        fn f1(_: ()) {}
        fn f2(_: ()) {}
        let h1 = Handler::new(f1);
        let h2 = Handler::new(f2);
        assert_ne!(h1, h2);
    }

    #[test]
    fn function_ptr_ne() {
        fn f1(_: ()) {}
        fn f2(_: ()) {}
        let h1 = Handler::new(f1 as fn(()) -> ());
        let h2 = Handler::new(f2 as fn(()) -> ());
        assert_ne!(h1, h2);
    }
}
