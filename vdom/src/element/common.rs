use super::{ApplyResult, Attribute, Diff, List, PatchAttribute, PatchList};
use serde::{
    de::{MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{
    cmp::Ordering,
    fmt::{self, Formatter},
};

#[derive(Default, Debug)]
pub struct Common<Msg> {
    pub(crate) key: Option<String>,
    pub(crate) attr: Vec<Attribute<Msg>>,
    pub(crate) children: List<Msg>,
}

impl<Msg> Common<Msg> {
    pub fn new(key: Option<String>, mut attr: Vec<Attribute<Msg>>, children: List<Msg>) -> Self {
        attr.sort_by(|a, b| a.attribute_type().cmp(&b.attribute_type()));
        attr.dedup_by(|a, b| a.attribute_type() == b.attribute_type());
        Self {
            key,
            attr,
            children,
        }
    }
}

impl<Msg> Diff for Common<Msg> {
    type Patch = PatchCommon<Msg>;
    fn diff(&self, other: &mut Self) -> Option<Self::Patch> {
        if self != other {
            let mut i1 = 0;
            let mut i2 = 0;
            let mut attr = vec![];
            while i1 < self.attr.len() && i2 < other.attr.len() {
                let this = &self.attr[i1];
                let other = &other.attr[i2];
                match this.attribute_type().cmp(&other.attribute_type()) {
                    Ordering::Less => {
                        i1 += 1;
                        attr.push(PatchAttribute::Remove(this.attribute_type()))
                    }
                    Ordering::Greater => {
                        i2 += 1;
                        attr.push(PatchAttribute::Insert(other.clone()))
                    }
                    Ordering::Equal => {
                        i1 += 1;
                        i2 += 1;
                        if this != other {
                            attr.push(PatchAttribute::Insert(other.clone()))
                        }
                    }
                }
            }
            while i1 < self.attr.len() {
                attr.push(PatchAttribute::Remove(self.attr[i1].attribute_type()));
                i1 += 1;
            }
            while i2 < other.attr.len() {
                attr.push(PatchAttribute::Insert(other.attr[i2].clone()));
                i2 += 1;
            }
            Some(PatchCommon {
                attr,
                children: self.children.diff(&mut other.children),
            })
        } else {
            None
        }
    }
    fn apply(&mut self, patch: Self::Patch) -> ApplyResult {
        let mut i = 0;
        for attr in patch.attr {
            match attr {
                PatchAttribute::Remove(attribute_type) => {
                    while self.attr[i].attribute_type() < attribute_type {
                        i += 1;
                    }
                    self.attr.remove(i);
                }
                PatchAttribute::Insert(attribute) => {
                    while i < self.attr.len()
                        && self.attr[i].attribute_type() < attribute.attribute_type()
                    {
                        i += 1;
                    }
                    if i == self.attr.len() {
                        self.attr.push(attribute);
                    } else if self.attr[i].attribute_type() == attribute.attribute_type() {
                        self.attr[i] = attribute;
                        i += 1;
                    } else {
                        self.attr.insert(i, attribute);
                        i += 1;
                    }
                }
            }
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
            attr: self.attr.clone(),
            children: self.children.clone(),
        }
    }
}

impl<Msg> PartialEq for Common<Msg> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.attr == other.attr && self.children == other.children
    }
}

impl<Msg> Eq for Common<Msg> {}

impl<Msg> Serialize for Common<Msg> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut r#struct = serializer.serialize_struct("Common", 2)?;
        r#struct.serialize_field("attr", &self.attr)?;
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
                let (_, attr) = map.next_entry::<&str, Vec<Attribute<Msg>>>()?.unwrap();
                let (_, children) = map.next_entry::<&str, List<Msg>>()?.unwrap();
                Ok(Common::new(None, attr, children))
            }
        }
        deserializer.deserialize_struct(
            "Common",
            &["id", "children"],
            CommonVisitor(Default::default()),
        )
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PatchCommon<Msg> {
    pub(crate) attr: Vec<PatchAttribute<Msg>>,
    pub(crate) children: Option<PatchList<Msg>>,
}

impl<Msg> Serialize for PatchCommon<Msg> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut r#struct = serializer.serialize_struct("PatchCommon", 2)?;
        r#struct.serialize_field("attr", &self.attr)?;
        r#struct.serialize_field("children", &self.children)?;
        r#struct.end()
    }
}

impl<'de, Msg> Deserialize<'de> for PatchCommon<Msg> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PatchCommonVisitor<Msg>(std::marker::PhantomData<Msg>);
        impl<'de, Msg> Visitor<'de> for PatchCommonVisitor<Msg> {
            type Value = PatchCommon<Msg>;
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                write!(formatter, "struct of id and children")
            }
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let (_, attr) = map.next_entry::<&str, Vec<PatchAttribute<Msg>>>()?.unwrap();
                let (_, children) = map.next_entry::<&str, Option<PatchList<Msg>>>()?.unwrap();
                Ok(PatchCommon { attr, children })
            }
        }
        deserializer.deserialize_struct(
            "PatchCommonVisitor",
            &["id", "children"],
            PatchCommonVisitor(Default::default()),
        )
    }
}

#[cfg(test)]
mod test {
    use super::{
        super::{id, PatchAttribute},
        Common, Diff, PatchCommon,
    };
    #[test]
    fn same() {
        let common1 = Common::<()>::default();
        let mut common2 = Common::default();
        assert_eq!(common1.diff(&mut common2), None)
    }

    #[test]
    fn different_id() {
        let mut common1 = Common::<()>::new(None, vec![id("a".into())], Default::default());
        let mut common2 = Common::new(None, vec![id("b".into())], Default::default());
        let patch = common1.diff(&mut common2);
        assert_eq!(
            patch,
            Some(PatchCommon {
                attr: vec![PatchAttribute::Insert(id("b".into()))],
                children: Default::default()
            })
        );
        common1.apply(patch.unwrap()).unwrap();
        assert_eq!(common1, common2);
    }
}
