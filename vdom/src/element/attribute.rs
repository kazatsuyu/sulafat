use crate::ClosureId;
use serde::{
    de::{EnumAccess, VariantAccess, Visitor},
    ser::SerializeTupleVariant,
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_derive::{Deserialize, Serialize};
use std::{
    any::Any,
    collections::HashMap,
    fmt::Debug,
    fmt::{self, Formatter},
    hash::Hash,
    rc::{Rc, Weak},
};
use sulafat_macros::with_types;

pub struct Handler<Args, Msg> {
    id: ClosureId,
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
            id: ClosureId::new::<F, Args, Msg>(&f),
            handle: Box::new(f),
        }
    }

    fn wrap<Msg2>(self) -> Handler<Args, Msg2>
    where
        Msg: 'static,
        Args: 'static,
        Msg2: 'static + From<Msg>,
    {
        Handler {
            id: self.id,
            handle: Box::new(move |args| self.handle.invoke(args).into()),
        }
    }

    fn as_any(self: Rc<Self>) -> Rc<dyn Any>
    where
        Msg: 'static,
        Args: 'static,
    {
        self as Rc<dyn Any>
    }
}

impl<Args, Msg> Debug for Handler<Args, Msg> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Handler({:?})", self.id)
    }
}

impl<Args, Msg> Serialize for Handler<Args, Msg> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_newtype_struct("Handler", &self.id)
    }
}

impl<'de, Args, Msg> Deserialize<'de> for Handler<Args, Msg> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct HandlerVisitor<Args, Msg>(std::marker::PhantomData<(Args, Msg)>);
        impl<'de, Args, Msg> Visitor<'de> for HandlerVisitor<Args, Msg> {
            type Value = Handler<Args, Msg>;
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                write!(formatter, "Id")
            }
            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                let id = ClosureId::deserialize(deserializer)?;
                Ok(Handler {
                    id,
                    handle: Box::new(|_| unreachable!()),
                })
            }
        }
        deserializer.deserialize_newtype_struct("Handler", HandlerVisitor(Default::default()))
    }
}

pub trait Handle<Args, Msg> {
    fn invoke(&self, args: Args) -> Msg;
}

impl<Msg, F, Args> Handle<Args, Msg> for F
where
    F: 'static + Fn(Args) -> Msg,
{
    fn invoke(&self, args: Args) -> Msg {
        self(args)
    }
}

#[with_types]
#[derive(Debug)]
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
                handlers.insert(handler.id, Rc::downgrade(&handler.clone().as_any()));
            }
            Attribute::OnPointerMove(handler) => {
                handlers.insert(handler.id, Rc::downgrade(&handler.clone().as_any()));
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
        self.id == other.id
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
                variant.serialize_field(&handler.id)?;
                variant.end()
            }
            Attribute::OnPointerMove(handler) => {
                let mut variant =
                    serializer.serialize_tuple_variant("Attribute", 2, "OnPointerMove", 1)?;
                variant.serialize_field(&handler.id)?;
                variant.end()
            }
        }
    }
}

impl<'de, Msg> Deserialize<'de> for Attribute<Msg> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AttributeVisitor<Msg>(std::marker::PhantomData<Msg>);

        impl<'de, Msg> Visitor<'de> for AttributeVisitor<Msg> {
            type Value = Attribute<Msg>;
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                write!(formatter, "variant of Single or List")
            }
            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                #[derive(Deserialize)]
                enum Tag {
                    Id,
                    OnClick,
                    OnPointerMove,
                }
                let (v, variant) = data.variant::<Tag>()?;
                Ok(match v {
                    Tag::Id => Attribute::Id(variant.newtype_variant::<String>()?),
                    Tag::OnClick => {
                        Attribute::OnClick(Rc::new(variant.newtype_variant::<Handler<(), Msg>>()?))
                    }
                    Tag::OnPointerMove => Attribute::OnPointerMove(Rc::new(
                        variant.newtype_variant::<Handler<(f64, f64), Msg>>()?,
                    )),
                })
            }
        }
        deserializer.deserialize_enum(
            "Node",
            &["Single", "List"],
            AttributeVisitor(Default::default()),
        )
    }
}

#[derive(Debug)]
pub enum PatchAttribute<Msg> {
    Remove(AttributeType),
    Insert(Attribute<Msg>),
}

impl<Msg> Clone for PatchAttribute<Msg> {
    fn clone(&self) -> Self {
        match self {
            PatchAttribute::Remove(attribute_type) => {
                PatchAttribute::Remove(attribute_type.clone())
            }
            PatchAttribute::Insert(attribute) => PatchAttribute::Insert(attribute.clone()),
        }
    }
}

impl<Msg> PartialEq for PatchAttribute<Msg> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PatchAttribute::Remove(this), PatchAttribute::Remove(other)) => this == other,
            (PatchAttribute::Insert(this), PatchAttribute::Insert(other)) => this == other,
            _ => false,
        }
    }
}

impl<Msg> Eq for PatchAttribute<Msg> {}

impl<Msg> Serialize for PatchAttribute<Msg> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            PatchAttribute::Remove(attribute_type) => {
                let mut variant =
                    serializer.serialize_tuple_variant("PatchAttributeOp", 0, "Remove", 1)?;
                variant.serialize_field(attribute_type)?;
                variant.end()
            }
            PatchAttribute::Insert(attribute) => {
                let mut variant =
                    serializer.serialize_tuple_variant("PatchAttributeOp", 0, "Insert", 1)?;
                variant.serialize_field(attribute)?;
                variant.end()
            }
        }
    }
}

impl<'de, Msg> Deserialize<'de> for PatchAttribute<Msg> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PatchAttributeOpVisitor<Msg>(std::marker::PhantomData<Msg>);
        impl<'de, Msg> Visitor<'de> for PatchAttributeOpVisitor<Msg> {
            type Value = PatchAttribute<Msg>;
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                write!(formatter, "variant of Remove or Insert")
            }
            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                #[derive(Deserialize)]
                enum Tag {
                    Remove,
                    Insert,
                }
                let (v, variant) = data.variant::<Tag>()?;
                Ok(match v {
                    Tag::Remove => {
                        PatchAttribute::Remove(variant.newtype_variant::<AttributeType>()?)
                    }
                    Tag::Insert => {
                        PatchAttribute::Insert(variant.newtype_variant::<Attribute<Msg>>()?)
                    }
                })
            }
        }

        deserializer.deserialize_enum(
            "PatchAttributeOp",
            &["Remove", "Insert"],
            PatchAttributeOpVisitor(Default::default()),
        )
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
