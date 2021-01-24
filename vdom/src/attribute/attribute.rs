use crate::{ClosureId, Handler};
use serde::{ser::SerializeTupleVariant, Serialize, Serializer};
use std::{
    any::Any,
    collections::HashMap,
    fmt::Debug,
    hash::Hash,
    rc::{Rc, Weak},
};
use sulafat_macros::VariantIdent;

use super::Style;

#[derive(Debug, VariantIdent)]
pub enum Attribute<Msg> {
    Id(String),
    OnClick(Rc<Handler<(), Msg>>),
    OnPointerMove(Rc<Handler<(f64, f64), Msg>>),
    Style(Style),
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
            Attribute::Style(_) => {}
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
            Attribute::Style(style) => Attribute::Style(style.clone()),
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
            Attribute::Style(style) => {
                let mut variant = serializer.serialize_tuple_variant("Attribute", 3, "Style", 1)?;
                variant.serialize_field(style)?;
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
