use super::{ApplyResult, Diff, List, PatchList};
use serde::{
    de::{MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_derive::{Deserialize, Serialize};
use std::fmt::{self, Formatter};

#[derive(Default, Debug)]
pub struct Common<Msg> {
    pub(crate) key: Option<String>,
    pub(crate) id: Option<String>,
    pub(crate) children: List<Msg>,
}

impl<Msg> Common<Msg> {
    pub fn new(key: Option<String>, id: Option<String>, children: List<Msg>) -> Self {
        Self { key, id, children }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PatchCommon<Msg> {
    pub(crate) id: Option<Option<String>>,
    pub(crate) children: Option<PatchList<Msg>>,
}

impl<Msg> Diff for Common<Msg> {
    type Patch = PatchCommon<Msg>;
    fn diff(&self, other: &Self) -> Option<Self::Patch> {
        if self != other {
            Some(PatchCommon {
                id: if self.id != other.id {
                    Some(other.id.clone())
                } else {
                    None
                },
                children: self.children.diff(&other.children),
            })
        } else {
            None
        }
    }
    fn apply(&mut self, patch: Self::Patch) -> ApplyResult {
        if let Some(id) = patch.id {
            self.id = id;
        }
        if let Some(patch) = patch.children {
            self.children.apply(patch)?;
        }
        Ok(())
    }
}

impl<Msg> Clone for Common<Msg> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            id: self.id.clone(),
            children: self.children.clone(),
        }
    }
}

impl<Msg> PartialEq for Common<Msg> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.id == other.id && self.children == other.children
    }
}

impl<Msg> Eq for Common<Msg> {}

impl<Msg> Serialize for Common<Msg> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut r#struct = serializer.serialize_struct("Common", 2)?;
        r#struct.serialize_field("id", &self.id)?;
        r#struct.serialize_field("children", &self.children)?;
        r#struct.end()
    }
}

impl<'de, Msg> Deserialize<'de> for Common<Msg> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CommonVisitor<Msg>(std::marker::PhantomData<Msg>);
        impl<'de, Msg> Visitor<'de> for CommonVisitor<Msg> {
            type Value = Common<Msg>;
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                write!(formatter, "struct of id and children")
            }
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let (_, id) = map.next_entry::<&str, Option<String>>()?.unwrap();
                let (_, children) = map.next_entry::<&str, List<Msg>>()?.unwrap();
                Ok(Common::new(None, id, children))
            }
        }
        deserializer.deserialize_struct(
            "Common",
            &["id", "children"],
            CommonVisitor(Default::default()),
        )
    }
}

#[cfg(test)]
mod test {
    use super::{Common, Diff, PatchCommon};
    #[test]
    fn common_same() {
        let common1 = Common::<()>::default();
        let common2 = Common::default();
        assert_eq!(common1.diff(&common2), None)
    }

    #[test]
    fn common_different_id() {
        let mut common1 = Common::<()>::new(None, Some("a".into()), Default::default());
        let common2 = Common::new(None, Some("b".into()), Default::default());
        let patch = common1.diff(&common2);
        assert_eq!(
            patch,
            Some(PatchCommon {
                id: Some(Some("b".into())),
                children: Default::default()
            })
        );
        common1.apply(patch.unwrap()).unwrap();
        assert_eq!(common1, common2);
    }
}
