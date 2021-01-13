use std::{any::Any, collections::HashMap, fmt, rc::Weak};

use crate::ClosureId;

use super::{ApplyResult, Diff, Element, Node, PatchElement, PatchNode};
use fmt::Formatter;
use serde::{
    de::{EnumAccess, VariantAccess, Visitor},
    ser::SerializeTupleVariant,
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_derive::Deserialize;

#[derive(Debug)]
pub enum Single<Msg> {
    Text(String),
    Element(Element<Msg>),
}

impl<Msg> Single<Msg> {
    pub fn key(&self) -> Option<&String> {
        if let Single::Element(element) = self {
            element.key().into()
        } else {
            None
        }
    }

    pub(crate) fn is_full_rendered(&self) -> bool {
        match self {
            Single::Text(_) => true,
            Single::Element(element) => element.is_full_rendered(),
        }
    }

    pub(crate) fn full_render(&mut self) {
        match self {
            Single::Text(_) => {}
            Single::Element(element) => element.full_render(),
        }
    }

    pub(crate) fn pick_handler(&self, handlers: &mut HashMap<ClosureId, Weak<dyn Any>>)
    where
        Msg: 'static,
    {
        match self {
            Single::Text(_) => {}
            Single::Element(element) => element.pick_handler(handlers),
        }
    }
}

impl<Msg> From<Single<Msg>> for Node<Msg> {
    fn from(node: Single<Msg>) -> Self {
        Node::Single(node)
    }
}

impl<Msg> From<PatchSingle<Msg>> for PatchNode<Msg> {
    fn from(patch: PatchSingle<Msg>) -> Self {
        match patch {
            PatchSingle::Replace(single) => PatchNode::Replace(single.into()),
            _ => PatchNode::Single(patch),
        }
    }
}

impl<Msg> Diff for Single<Msg> {
    type Patch = PatchSingle<Msg>;
    fn diff(&self, other: &mut Self) -> Option<Self::Patch> {
        match (self, other) {
            (Single::Text(s), Single::Text(o)) => {
                if s == o {
                    None
                } else {
                    Some(PatchSingle::Replace(Single::Text(o.clone())))
                }
            }
            (Single::Element(s), Single::Element(o)) => Some(s.diff(o)?.into()),
            (_, other) => Some(PatchSingle::Replace(other.clone())),
        }
    }
    fn apply(&mut self, patch: Self::Patch) -> ApplyResult {
        match patch {
            PatchSingle::Replace(node) => *self = node,
            PatchSingle::Element(patch) => {
                if let Single::Element(element) = self {
                    element.apply(patch)?
                } else {
                    return Err("Elementではありません".into());
                }
            }
        }
        Ok(())
    }
}

impl<Msg> From<String> for Single<Msg> {
    fn from(s: String) -> Self {
        Single::Text(s)
    }
}

impl<Msg> From<&str> for Single<Msg> {
    fn from(s: &str) -> Self {
        Single::Text(s.into())
    }
}

impl<Msg> From<String> for Node<Msg> {
    fn from(s: String) -> Self {
        Single::from(s).into()
    }
}

impl<Msg> From<&str> for Node<Msg> {
    fn from(s: &str) -> Self {
        Single::from(s).into()
    }
}

impl<Msg> Serialize for Single<Msg> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Single::Text(s) => {
                let mut variant = serializer.serialize_tuple_variant("Single", 0, "Text", 1)?;
                variant.serialize_field(s)?;
                variant.end()
            }
            Single::Element(e) => {
                let mut variant = serializer.serialize_tuple_variant("Single", 1, "Element", 1)?;
                variant.serialize_field(e)?;
                variant.end()
            }
        }
    }
}

impl<'de, Msg> Deserialize<'de> for Single<Msg> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SingleVisitor<Msg>(std::marker::PhantomData<Msg>);
        impl<'de, Msg> Visitor<'de> for SingleVisitor<Msg> {
            type Value = Single<Msg>;
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                write!(formatter, "variant of Text or Element")
            }
            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                #[derive(Debug, Copy, Clone, Deserialize)]
                enum VariantTag {
                    Text,
                    Element,
                }
                let (v, variant) = data.variant::<VariantTag>()?;
                Ok(match v {
                    VariantTag::Text => variant.newtype_variant::<String>()?.into(),
                    VariantTag::Element => variant.newtype_variant::<Element<Msg>>()?.into(),
                })
            }
        }
        deserializer.deserialize_enum(
            "Single",
            &["Text", "Element"],
            SingleVisitor(Default::default()),
        )
    }
}

impl<Msg> Clone for Single<Msg> {
    fn clone(&self) -> Self {
        match self {
            Single::Text(s) => Single::Text(s.clone()),
            Single::Element(e) => Single::Element(e.clone()),
        }
    }
}

impl<Msg> PartialEq for Single<Msg> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Single::Text(this), Single::Text(other)) => this == other,
            (Single::Element(this), Single::Element(other)) => this == other,
            _ => false,
        }
    }
}

impl<Msg> Eq for Single<Msg> {}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PatchSingle<Msg> {
    Replace(Single<Msg>),
    Element(PatchElement<Msg>),
}

impl<Msg> Serialize for PatchSingle<Msg> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            PatchSingle::Replace(single) => {
                let mut variant =
                    serializer.serialize_tuple_variant("PatchSingle", 0, "Replace", 1)?;
                variant.serialize_field(single)?;
                variant.end()
            }
            PatchSingle::Element(patch) => {
                let mut variant =
                    serializer.serialize_tuple_variant("PatchSingle", 1, "Element", 1)?;
                variant.serialize_field(patch)?;
                variant.end()
            }
        }
    }
}

impl<'de, Msg> Deserialize<'de> for PatchSingle<Msg> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PatchSingleVisitor<Msg>(std::marker::PhantomData<Msg>);

        impl<'de, Msg> Visitor<'de> for PatchSingleVisitor<Msg> {
            type Value = PatchSingle<Msg>;
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                write!(formatter, "variant of Replace or Element")
            }
            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                #[derive(Deserialize)]
                enum VariantTag {
                    Replace,
                    Element,
                }
                let (v, variant) = data.variant::<VariantTag>()?;
                Ok(match v {
                    VariantTag::Replace => PatchSingle::Replace(variant.newtype_variant()?),
                    VariantTag::Element => PatchSingle::Element(variant.newtype_variant()?),
                })
            }
        }
        deserializer.deserialize_enum(
            "PatchSingle",
            &["Replace", "Element"],
            PatchSingleVisitor(Default::default()),
        )
    }
}

#[cfg(test)]
mod test {
    use super::{super::Div, Diff, PatchSingle, Single};

    #[test]
    fn same_element() {
        let div1: Single<()> = Div::default().into();
        let mut div2 = Div::default().into();
        assert_eq!(div1.diff(&mut div2), None);
    }

    #[test]
    fn same_text() {
        let text1: Single<()> = "a".into();
        let mut text2 = "a".into();
        assert_eq!(text1.diff(&mut text2), None);
    }

    #[test]
    fn different() {
        let mut text: Single<()> = "a".into();
        let mut div = Div::default().into();
        assert_ne!(text, div);
        let patch = text.diff(&mut div);
        assert_eq!(patch, Some(PatchSingle::Replace(div.clone())));
        text.apply(patch.unwrap()).unwrap();
        assert_eq!(text, div);
    }
}
