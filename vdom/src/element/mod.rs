mod common;
mod div;
mod span;

pub use common::{Common, PatchCommon};
pub use div::{Div, PatchDiv};
pub use span::{PatchSpan, Span};

use super::{ApplyResult, Diff, List, Node, PatchList, PatchNode, PatchSingle, Single};

use serde::{
    de::{EnumAccess, VariantAccess, Visitor},
    ser::SerializeTupleVariant,
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_derive::{Deserialize, Serialize};
use std::fmt::{self, Formatter};
use sulafat_macros::with_types;

#[with_types]
#[derive(Debug, Eq)]
pub enum Element<Msg> {
    Div(Div<Msg>),
    Span(Span<Msg>),
}

impl<Msg> Element<Msg> {
    fn common(&self) -> &Common<Msg> {
        match self {
            Element::Div(div) => div.common(),
            Element::Span(span) => span.common(),
        }
    }
    fn common_mut(&mut self) -> &mut Common<Msg> {
        match self {
            Element::Div(div) => div.common_mut(),
            Element::Span(span) => span.common_mut(),
        }
    }

    pub fn id(&self) -> &Option<String> {
        &self.common().id
    }

    pub fn children(&self) -> &List<Msg> {
        &self.common().children
    }

    pub fn children_mut(&mut self) -> &mut List<Msg> {
        &mut self.common_mut().children
    }

    pub fn key(&self) -> &Option<String> {
        &self.common().key
    }
}

impl<Msg> From<Element<Msg>> for Single<Msg> {
    fn from(element: Element<Msg>) -> Self {
        Single::Element(element)
    }
}

impl<Msg> From<Element<Msg>> for Node<Msg> {
    fn from(element: Element<Msg>) -> Self {
        Single::from(element).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum PatchElement<Msg> {
    Replace(Element<Msg>),
    Div(PatchDiv<Msg>),
    Span(PatchSpan<Msg>),
}

impl<Msg> From<PatchElement<Msg>> for PatchSingle<Msg> {
    fn from(patch: PatchElement<Msg>) -> Self {
        match patch {
            PatchElement::Replace(single) => PatchSingle::Replace(single.into()),
            _ => PatchSingle::Element(patch),
        }
    }
}

impl<Msg> From<PatchElement<Msg>> for PatchNode<Msg> {
    fn from(patch: PatchElement<Msg>) -> Self {
        PatchSingle::from(patch).into()
    }
}

impl<Msg> Diff for Element<Msg> {
    type Patch = PatchElement<Msg>;
    fn diff(&self, other: &Self) -> Option<Self::Patch> {
        use Element::*;
        if self.key() != other.key() {
            return Some(PatchElement::Replace(other.clone()));
        }
        Some(match (self, other) {
            (Div(div1), Div(div2)) => div1.diff(div2)?.into(),
            (Span(div1), Span(div2)) => div1.diff(div2)?.into(),
            _ => PatchElement::Replace(other.clone()),
        })
    }

    fn apply(&mut self, patch: Self::Patch) -> ApplyResult {
        use PatchElement::*;
        macro_rules! apply_element {
            ($($id:ident),*) => {
                match patch {
                    Replace(element) => {
                        *self = element;
                        return Ok(());
                    }
                    $($id(patch) => {
                        if let Element::$id(element) = self {
                            return element.apply(patch);
                        }
                    })*
                }
            };
        }
        apply_element!(Div, Span);
        Err("異なる要素です".into())
    }
}

impl<Msg> Clone for Element<Msg> {
    fn clone(&self) -> Self {
        match self {
            Element::Div(div) => Element::Div(div.clone()),
            Element::Span(span) => Element::Span(span.clone()),
        }
    }
}

impl<Msg> PartialEq for Element<Msg> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Element::Div(this), Element::Div(other)) => this == other,
            (Element::Span(this), Element::Span(other)) => this == other,
            _ => false,
        }
    }
}

impl<Msg> Serialize for Element<Msg> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Element::Div(div) => {
                let mut variant = serializer.serialize_tuple_variant("Element", 0, "Div", 1)?;
                variant.serialize_field(div)?;
                variant.end()
            }
            Element::Span(span) => {
                let mut variant = serializer.serialize_tuple_variant("Element", 1, "Span", 1)?;
                variant.serialize_field(span)?;
                variant.end()
            }
        }
    }
}

impl<'de, Msg> Deserialize<'de> for Element<Msg> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ElementVisitor<Msg>(std::marker::PhantomData<Msg>);
        impl<'de, Msg> Visitor<'de> for ElementVisitor<Msg> {
            type Value = Element<Msg>;
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                write!(formatter, "variant of Div or Span")
            }
            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                #[derive(Debug, Copy, Clone, Deserialize)]
                enum VariantTag {
                    Div,
                    Span,
                }
                let (v, variant) = data.variant::<VariantTag>()?;
                Ok(match v {
                    VariantTag::Div => variant.newtype_variant::<Div<Msg>>()?.into(),
                    VariantTag::Span => variant.newtype_variant::<Span<Msg>>()?.into(),
                })
            }
        }
        deserializer.deserialize_enum(
            "Element",
            &["Div", "Span"],
            ElementVisitor(Default::default()),
        )
    }
}

#[cfg(test)]
mod test {
    use super::{Common, Diff, Div, Element, PatchCommon, PatchDiv, PatchSpan, Span};

    #[test]
    fn div_same() {
        let div1: Element<()> = Div::default().into();
        let div2 = Div::default().into();
        assert_eq!(div1.diff(&div2), None);
    }

    #[test]
    fn span_same() {
        let span1: Element<()> = Span::default().into();
        let span2 = Span::default().into();
        assert_eq!(span1.diff(&span2), None);
    }

    #[test]
    fn div_different_id() {
        let mut div1: Element<()> =
            Div::new(Common::new(None, Some("a".into()), Default::default())).into();
        let div2 = Div::new(Common::new(None, Some("b".into()), Default::default())).into();
        assert_ne!(div1, div2);
        let patch = div1.diff(&div2);
        assert_eq!(
            patch,
            Some(
                PatchDiv {
                    common: PatchCommon {
                        id: Some(Some("b".into())),
                        children: Default::default()
                    }
                }
                .into()
            )
        );
        div1.apply(patch.unwrap()).unwrap();
        assert_eq!(div1, div2);
    }

    #[test]
    fn span_different_id() {
        let mut span1: Element<()> =
            Span::new(Common::new(None, Some("a".into()), Default::default())).into();
        let span2 = Span::new(Common::new(None, Some("b".into()), Default::default())).into();
        assert_ne!(span1, span2);
        let patch = span1.diff(&span2);
        assert_eq!(
            patch,
            Some(
                PatchSpan {
                    common: PatchCommon {
                        id: Some(Some("b".into())),
                        children: Default::default()
                    }
                }
                .into()
            )
        );
        span1.apply(patch.unwrap()).unwrap();
        assert_eq!(span1, span2);
    }

    #[test]
    fn div_different_key() {
        let mut div1: Element<()> =
            Div::new(Common::new(Some("a".into()), None, Default::default())).into();
        let div2 = Div::new(Common::new(Some("b".into()), None, Default::default())).into();
        assert_ne!(div1, div2);
        let patch = div1.diff(&div2);
        assert_eq!(patch, Some(super::PatchElement::Replace(div2.clone())));
        div1.apply(patch.unwrap()).unwrap();
        assert_eq!(div1, div2);
    }

    #[test]
    fn span_different_key() {
        let mut span1: Element<()> =
            Span::new(Common::new(Some("a".into()), None, Default::default())).into();
        let span2 = Span::new(Common::new(Some("b".into()), None, Default::default())).into();
        assert_ne!(span1, span2);
        let patch = span1.diff(&span2);
        assert_eq!(patch, Some(super::PatchElement::Replace(span2.clone())));
        span1.apply(patch.unwrap()).unwrap();
        assert_eq!(span1, span2);
    }

    #[test]
    fn element_different_tag() {
        let mut div: Element<()> = Div::default().into();
        let span = Span::default().into();
        assert_ne!(div, span);
        let patch = div.diff(&span);
        assert_eq!(patch, Some(super::PatchElement::Replace(span.clone())));
        div.apply(patch.unwrap()).unwrap();
        assert_eq!(div, span);
    }
}
