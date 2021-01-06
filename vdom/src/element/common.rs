use super::{ApplyResult, Attribute, AttributeType, Diff, List, PatchList};
use serde::{
    de::{EnumAccess, MapAccess, VariantAccess, Visitor},
    ser::{SerializeStruct, SerializeTupleVariant},
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_derive::{Deserialize, Serialize};
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

#[derive(Debug)]
pub enum PatchAttributeOp<Msg> {
    Remove(AttributeType),
    Insert(Attribute<Msg>),
}

impl<Msg> Clone for PatchAttributeOp<Msg> {
    fn clone(&self) -> Self {
        match self {
            PatchAttributeOp::Remove(attribute_type) => {
                PatchAttributeOp::Remove(attribute_type.clone())
            }
            PatchAttributeOp::Insert(attribute) => PatchAttributeOp::Insert(attribute.clone()),
        }
    }
}

impl<Msg> PartialEq for PatchAttributeOp<Msg> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PatchAttributeOp::Remove(this), PatchAttributeOp::Remove(other)) => this == other,
            (PatchAttributeOp::Insert(this), PatchAttributeOp::Insert(other)) => this == other,
            _ => false,
        }
    }
}

impl<Msg> Serialize for PatchAttributeOp<Msg> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            PatchAttributeOp::Remove(attribute_type) => {
                let mut variant =
                    serializer.serialize_tuple_variant("PatchAttributeOp", 0, "Remove", 1)?;
                variant.serialize_field(attribute_type)?;
                variant.end()
            }
            PatchAttributeOp::Insert(attribute) => {
                let mut variant =
                    serializer.serialize_tuple_variant("PatchAttributeOp", 0, "Insert", 1)?;
                variant.serialize_field(attribute)?;
                variant.end()
            }
        }
    }
}

impl<'de, Msg> Deserialize<'de> for PatchAttributeOp<Msg> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PatchAttributeOpVisitor<Msg>(std::marker::PhantomData<Msg>);
        impl<'de, Msg> Visitor<'de> for PatchAttributeOpVisitor<Msg> {
            type Value = PatchAttributeOp<Msg>;
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
                        PatchAttributeOp::Remove(variant.newtype_variant::<AttributeType>()?)
                    }
                    Tag::Insert => {
                        PatchAttributeOp::Insert(variant.newtype_variant::<Attribute<Msg>>()?)
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

impl<Msg> Eq for PatchAttributeOp<Msg> {}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PatchAttribute<Msg>(Vec<PatchAttributeOp<Msg>>);

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PatchCommon<Msg> {
    pub(crate) attr: Vec<PatchAttributeOp<Msg>>,
    pub(crate) children: Option<PatchList<Msg>>,
}

impl<Msg> Diff for Common<Msg> {
    type Patch = PatchCommon<Msg>;
    fn diff(&self, other: &Self) -> Option<Self::Patch> {
        if self != other {
            let mut i1 = 0;
            let mut i2 = 0;
            let mut attr = vec![];
            while i1 < self.attr.len() && i2 < other.attr.len() {
                let this = &self.attr[i1];
                let other = &other.attr[i2];
                if this != other {
                    attr.push(PatchAttributeOp::Insert(other.clone()));
                }
                match this.attribute_type().cmp(&other.attribute_type()) {
                    Ordering::Less => {
                        i1 += 1;
                        attr.push(PatchAttributeOp::Remove(other.attribute_type()))
                    }
                    Ordering::Greater => {
                        i2 += 1;
                        attr.push(PatchAttributeOp::Insert(other.clone()))
                    }
                    Ordering::Equal => {
                        i1 += 1;
                        i2 += 1
                    }
                }
            }
            Some(PatchCommon {
                attr,
                children: self.children.diff(&other.children),
            })
        } else {
            None
        }
    }
    fn apply(&mut self, patch: Self::Patch) -> ApplyResult {
        let mut i = 0;
        for attr in patch.attr {
            match attr {
                PatchAttributeOp::Remove(attribute_type) => {
                    while self.attr[i].attribute_type() < attribute_type {
                        i += 1;
                    }
                    self.attr.remove(i);
                }
                PatchAttributeOp::Insert(attribute) => {
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

#[cfg(test)]
mod test {
    use super::{
        super::{id, PatchAttributeOp},
        Common, Diff, PatchCommon,
    };
    #[test]
    fn same() {
        let common1 = Common::<()>::default();
        let common2 = Common::default();
        assert_eq!(common1.diff(&common2), None)
    }

    #[test]
    fn different_id() {
        let mut common1 = Common::<()>::new(None, vec![id("a".into())], Default::default());
        let common2 = Common::new(None, vec![id("b".into())], Default::default());
        let patch = common1.diff(&common2);
        assert_eq!(
            patch,
            Some(PatchCommon {
                attr: vec![PatchAttributeOp::Insert(id("b".into()))],
                children: Default::default()
            })
        );
        common1.apply(patch.unwrap()).unwrap();
        assert_eq!(common1, common2);
    }
}
