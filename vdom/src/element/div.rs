use std::fmt;

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
use serde_derive::{Deserialize, Serialize};

#[derive(Default, Debug)]
pub struct Div<Msg> {
    pub(crate) common: Common<Msg>,
}

impl<Msg> Div<Msg> {
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

impl<Msg> From<Div<Msg>> for Element<Msg> {
    fn from(div: Div<Msg>) -> Self {
        Element::Div(div)
    }
}

impl<Msg> From<Div<Msg>> for Single<Msg> {
    fn from(div: Div<Msg>) -> Self {
        Element::from(div).into()
    }
}

impl<Msg> From<Div<Msg>> for Node<Msg> {
    fn from(div: Div<Msg>) -> Self {
        Single::from(div).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PatchDiv<Msg> {
    pub(crate) common: PatchCommon<Msg>,
}

impl<Msg> From<PatchDiv<Msg>> for PatchElement<Msg> {
    fn from(patch: PatchDiv<Msg>) -> Self {
        PatchElement::Div(patch)
    }
}

impl<Msg> From<PatchDiv<Msg>> for PatchSingle<Msg> {
    fn from(patch: PatchDiv<Msg>) -> Self {
        PatchElement::from(patch).into()
    }
}

impl<Msg> From<PatchDiv<Msg>> for PatchNode<Msg> {
    fn from(patch: PatchDiv<Msg>) -> Self {
        PatchSingle::from(patch).into()
    }
}

impl<Msg> Diff for Div<Msg> {
    type Patch = PatchDiv<Msg>;
    fn diff(&self, other: &Self) -> Option<Self::Patch> {
        Some(PatchDiv {
            common: self.common.diff(&other.common)?,
        })
    }
    fn apply(&mut self, patch: Self::Patch) -> ApplyResult {
        self.common.apply(patch.common)
    }
}

impl<Msg> Clone for Div<Msg> {
    fn clone(&self) -> Self {
        Self {
            common: self.common.clone(),
        }
    }
}

impl<Msg> PartialEq for Div<Msg> {
    fn eq(&self, other: &Self) -> bool {
        self.common == other.common
    }
}

impl<Msg> Eq for Div<Msg> {}

impl<Msg> Serialize for Div<Msg> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut r#struct = serializer.serialize_struct("Div", 1)?;
        r#struct.serialize_field("common", &self.common)?;
        r#struct.end()
    }
}

impl<'de, Msg> Deserialize<'de> for Div<Msg> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DivVisitor<Msg>(std::marker::PhantomData<Msg>);
        impl<'de, Msg> Visitor<'de> for DivVisitor<Msg> {
            type Value = Div<Msg>;
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                write!(formatter, "struct of common")
            }
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let (_, common) = map.next_entry::<&str, Common<Msg>>()?.unwrap();
                Ok(Div::new(common))
            }
        }
        deserializer.deserialize_struct("Div", &["common"], DivVisitor(Default::default()))
    }
}

#[cfg(test)]
mod test {
    use super::{
        super::{id, PatchAttributeOp},
        Common, Diff, Div, PatchCommon, PatchDiv,
    };
    #[test]
    fn same() {
        let div1 = Div::<()>::default();
        let div2 = Div::default();
        assert_eq!(div1.diff(&div2), None)
    }

    #[test]
    fn different_id() {
        let mut div1 = Div::<()>::new(Common::new(None, vec![id("a".into())], Default::default()));
        let div2 = Div::new(Common::new(None, vec![id("b".into())], Default::default()));
        assert_ne!(div1, div2);
        let patch = div1.diff(&div2);
        assert_eq!(
            patch,
            Some(PatchDiv {
                common: PatchCommon {
                    attr: vec![PatchAttributeOp::Insert(id("b".into()))],
                    children: Default::default()
                }
            })
        );
        div1.apply(patch.unwrap()).unwrap();
        assert_eq!(div1, div2);
    }
}
