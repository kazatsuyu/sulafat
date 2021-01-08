use crate::util::{reinterpret_cast, safe_cast, RawState, RawStateOf, StealHasher};
use serde::{
    de::{EnumAccess, VariantAccess, Visitor},
    ser::SerializeTupleVariant,
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_derive::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    fmt::{self, Formatter},
    hash::Hash,
    rc::Rc,
};
use sulafat_macros::with_types;

pub struct Handler<Args, Msg> {
    id: Id,
    handle: Rc<dyn Handle<Args, Msg>>,
}

const SIZE_OF_TYPEID: usize = std::mem::size_of::<std::any::TypeId>();

type TypeIdRawType = <RawStateOf<SIZE_OF_TYPEID> as RawState>::Type;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub struct TypeId {
    t: TypeIdRawType,
}

impl TypeId {
    fn of<T: 'static>() -> Self {
        let type_id = std::any::TypeId::of::<T>();
        let mut hasher = StealHasher::<SIZE_OF_TYPEID>::default();
        type_id.hash(&mut hasher);
        Self { t: hasher.get() }
    }
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Id {
    TypeId(TypeId),
    FnPtr(usize),
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
        let id = TypeId::of::<F>();
        Self {
            id: if let Some(fn_ptr) = safe_cast::<F, fn(Args) -> Msg>(&f) {
                Id::FnPtr(*fn_ptr as usize)
            } else {
                Id::TypeId(id)
            },
            handle: Rc::new(f),
        }
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
                let id = Id::deserialize(deserializer)?;
                Ok(Handler {
                    id,
                    handle: Rc::new(|_| unreachable!()),
                })
            }
        }
        deserializer.deserialize_newtype_struct("Handler", HandlerVisitor(Default::default()))
    }
}

pub trait Convert<T> {
    fn convert(self) -> T;
}

impl<Args, Msg1, Msg2> Convert<Handler<Args, Msg2>> for Handler<Args, Msg1>
where
    Msg1: 'static + Into<Msg2>,
    Msg2: 'static,
    Args: 'static,
{
    fn convert(self) -> Handler<Args, Msg2> {
        Handler {
            id: self.id,
            handle: self.handle.convert(),
        }
    }
}

pub trait Handle<Args, Msg> {
    fn invoke(&self, args: Args) -> Msg;
}

impl<Args, Msg1, Msg2> Convert<Rc<dyn Handle<Args, Msg2>>> for Rc<dyn Handle<Args, Msg1>>
where
    Msg1: 'static + Into<Msg2>,
    Msg2: 'static,
    Args: 'static,
{
    fn convert(self) -> Rc<dyn Handle<Args, Msg2>> {
        if TypeId::of::<Msg1>() == TypeId::of::<Msg2>() {
            unsafe { reinterpret_cast(self) }
        } else {
            Rc::new(move |args| self.invoke(args).into())
        }
    }
}

impl<Msg, F, Args> Handle<Args, Msg> for F
where
    F: Fn(Args) -> Msg,
{
    fn invoke(&self, args: Args) -> Msg {
        self(args)
    }
}

#[with_types(AttributeType)]
#[derive(Debug)]
pub enum Attribute<Msg> {
    Id(String),
    OnClick(Handler<(), Msg>),
    OnPointerMove(Handler<(f64, f64), Msg>),
}

impl<Msg1, Msg2> Convert<Attribute<Msg2>> for Attribute<Msg1>
where
    Msg1: 'static + Into<Msg2>,
    Msg2: 'static,
{
    fn convert(self) -> Attribute<Msg2> {
        match self {
            Attribute::Id(id) => Attribute::Id(id),
            Attribute::OnClick(handler) => Attribute::OnClick(handler.convert()),
            Attribute::OnPointerMove(handler) => Attribute::OnPointerMove(handler.convert()),
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
    Attribute::OnClick(Handler::new(f))
}

pub fn on_pointer_move<Msg, F>(f: F) -> Attribute<Msg>
where
    F: 'static + Fn((f64, f64)) -> Msg,
    Msg: 'static,
{
    Attribute::OnPointerMove(Handler::new(f))
}

impl<Args, Msg> PartialEq for Handler<Args, Msg> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<Args, Msg> Eq for Handler<Args, Msg> {}

impl<Args, Msg> Clone for Handler<Args, Msg> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            handle: self.handle.clone(),
        }
    }
}

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
                variant.serialize_field(handler)?;
                variant.end()
            }
            Attribute::OnPointerMove(handler) => {
                let mut variant =
                    serializer.serialize_tuple_variant("Attribute", 2, "OnPointerMove", 1)?;
                variant.serialize_field(handler)?;
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
                        Attribute::OnClick(variant.newtype_variant::<Handler<(), Msg>>()?)
                    }
                    Tag::OnPointerMove => Attribute::OnPointerMove(
                        variant.newtype_variant::<Handler<(f64, f64), Msg>>()?,
                    ),
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
