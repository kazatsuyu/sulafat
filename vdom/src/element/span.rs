use std::{any::Any, collections::HashMap, fmt, rc::Weak};

use crate::ClosureId;

use super::{
    ApplyResult, Common, Diff, Element, Node, PatchCommon, PatchElement, PatchNode, PatchSingle,
    Single,
};
use fmt::Formatter;
use serde::{
    de::{MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};

#[derive(Default, Debug)]
pub struct Span<Msg> {
    pub(crate) common: Common<Msg>,
}

impl<Msg> Span<Msg> {
    pub fn new(common: Common<Msg>) -> Self {
        Self { common }
    }

    pub fn common(&self) -> &Common<Msg> {
        &self.common
    }

    pub fn common_mut(&mut self) -> &mut Common<Msg> {
        &mut self.common
    }
}

impl<Msg> From<Span<Msg>> for Element<Msg> {
    fn from(span: Span<Msg>) -> Self {
        Element::Span(span)
    }
}

impl<Msg> From<Span<Msg>> for Single<Msg> {
    fn from(span: Span<Msg>) -> Self {
        Element::from(span).into()
    }
}

impl<Msg> From<Span<Msg>> for Node<Msg> {
    fn from(span: Span<Msg>) -> Self {
        Single::from(span).into()
    }
}

impl<Msg> From<PatchSpan<Msg>> for PatchElement<Msg> {
    fn from(patch: PatchSpan<Msg>) -> Self {
        PatchElement::Span(patch)
    }
}

impl<Msg> From<PatchSpan<Msg>> for PatchSingle<Msg> {
    fn from(patch: PatchSpan<Msg>) -> Self {
        PatchElement::from(patch).into()
    }
}

impl<Msg> From<PatchSpan<Msg>> for PatchNode<Msg> {
    fn from(patch: PatchSpan<Msg>) -> Self {
        PatchSingle::from(patch).into()
    }
}

impl<Msg> Diff for Span<Msg> {
    type Patch = PatchSpan<Msg>;
    fn diff(&self, other: &mut Self) -> Option<Self::Patch> {
        Some(PatchSpan {
            common: self.common.diff(&mut other.common)?,
        })
    }
    fn apply(&mut self, patch: Self::Patch) -> ApplyResult {
        self.common.apply(patch.common)
    }
}

impl<Msg> Clone for Span<Msg> {
    fn clone(&self) -> Self {
        Self {
            common: self.common.clone(),
        }
    }
}

impl<Msg> PartialEq for Span<Msg> {
    fn eq(&self, other: &Self) -> bool {
        self.common == other.common
    }
}

impl<Msg> Eq for Span<Msg> {}

impl<Msg> Serialize for Span<Msg> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut r#struct = serializer.serialize_struct("Span", 1)?;
        r#struct.serialize_field("common", &self.common)?;
        r#struct.end()
    }
}

impl<'de, Msg> Deserialize<'de> for Span<Msg> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SpanVisitor<Msg>(std::marker::PhantomData<Msg>);
        impl<'de, Msg> Visitor<'de> for SpanVisitor<Msg> {
            type Value = Span<Msg>;
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                write!(formatter, "struct of common")
            }
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let (_, common) = map.next_entry::<&str, Common<Msg>>()?.unwrap();
                Ok(Span::new(common))
            }
        }
        deserializer.deserialize_struct("Span", &["common"], SpanVisitor(Default::default()))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PatchSpan<Msg> {
    pub(crate) common: PatchCommon<Msg>,
}

impl<Msg> PatchSpan<Msg> {
    pub(crate) fn pick_handler(&self, handlers: &mut HashMap<ClosureId, Weak<dyn Any>>)
    where
        Msg: 'static,
    {
        self.common.pick_handler(handlers);
    }
}

impl<Msg> Serialize for PatchSpan<Msg> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut r#struct = serializer.serialize_struct("PatchSpan", 1)?;
        r#struct.serialize_field("common", &self.common)?;
        r#struct.end()
    }
}

impl<'de, Msg> Deserialize<'de> for PatchSpan<Msg> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PatchSpanVisitor<Msg>(std::marker::PhantomData<Msg>);
        impl<'de, Msg> Visitor<'de> for PatchSpanVisitor<Msg> {
            type Value = PatchSpan<Msg>;
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                write!(formatter, "struct of common")
            }
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let (_, common) = map.next_entry::<&str, PatchCommon<Msg>>()?.unwrap();
                Ok(PatchSpan { common })
            }
        }
        deserializer.deserialize_struct(
            "PatchDiv",
            &["common"],
            PatchSpanVisitor(Default::default()),
        )
    }
}

#[cfg(test)]
mod test {
    use super::{
        super::{id, PatchAttribute},
        Common, Diff, PatchCommon, PatchSpan, Span,
    };
    #[test]
    fn same() {
        let span1 = Span::<()>::default();
        let mut span2 = Span::default();
        assert_eq!(span1.diff(&mut span2), None)
    }

    #[test]
    fn different_id() {
        let mut span1 =
            Span::<()>::new(Common::new(None, vec![id("a".into())], Default::default()));
        let mut span2 = Span::new(Common::new(None, vec![id("b".into())], Default::default()));
        assert_ne!(span1, span2);
        let patch = span1.diff(&mut span2);
        assert_eq!(
            patch,
            Some(PatchSpan {
                common: PatchCommon {
                    attr: vec![PatchAttribute::Insert(id("b".into()))],
                    children: Default::default()
                }
            })
        );
        span1.apply(patch.unwrap()).unwrap();
        assert_eq!(span1, span2);
    }
}
