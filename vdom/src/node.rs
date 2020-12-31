use std::fmt;

use super::{ApplyResult, ComponentNode, Diff, List, PatchList, PatchSingle, Single};
use fmt::Formatter;
use serde::{
    de::{EnumAccess, VariantAccess, Visitor},
    ser::SerializeTupleVariant,
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Node {
    Single(Single),
    List(List),
    Component(ComponentNode),
}

impl Node {
    pub fn key(&self) -> Option<&String> {
        if let Node::Single(single) = self {
            single.key()
        } else {
            None
        }
    }

    pub(super) fn flat_len(&self) -> usize {
        match self {
            Node::Single(_) => 1,
            Node::List(list) => list.flat_len(),
            Node::Component(_) => todo!(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum PatchNode {
    Replace(Node),
    Single(PatchSingle),
    List(PatchList),
}

impl Diff for Node {
    type Patch = PatchNode;
    fn diff(&self, other: &Self) -> Option<Self::Patch> {
        use Node::*;
        match (self, other) {
            (Single(s), Single(o)) => Some(s.diff(o)?.into()),
            (List(s), List(o)) => Some(s.diff(o)?.into()),
            _ => Some(PatchNode::Replace(other.clone())),
        }
    }

    fn apply(&mut self, patch: Self::Patch) -> ApplyResult {
        use PatchNode::*;
        match patch {
            Replace(node) => *self = node,
            Single(patch) => {
                if let Node::Single(single) = self {
                    single.apply(patch)?
                } else {
                    return Err("単一ノードではありません。".into());
                }
            }
            List(patch) => {
                if let Node::List(list) = self {
                    list.apply(patch)?
                } else {
                    return Err("リストではありません。".into());
                }
            }
        }
        Ok(())
    }
}

impl Serialize for Node {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Node::Single(single) => {
                let mut tv = serializer.serialize_tuple_variant("Node", 0, "Single", 1)?;
                tv.serialize_field(single)?;
                tv.end()
            }
            Node::List(list) => {
                let mut tv = serializer.serialize_tuple_variant("Node", 1, "List", 1)?;
                tv.serialize_field(list)?;
                tv.end()
            }
            Node::Component(_) => todo!(),
        }
    }
}

struct NodeVisitor;

impl<'de> Visitor<'de> for NodeVisitor {
    type Value = Node;
    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "variant of Single or List")
    }
    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: EnumAccess<'de>,
    {
        #[derive(Deserialize, Debug)]
        enum VariantTag {
            Single,
            List,
        }
        let (v, variant) = data.variant::<VariantTag>()?;
        Ok(match v {
            VariantTag::Single => variant.newtype_variant::<Single>()?.into(),
            VariantTag::List => variant.newtype_variant::<List>()?.into(),
        })
    }
}

impl<'de> Deserialize<'de> for Node {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_enum("Node", &["Single", "List"], NodeVisitor)
    }
}

#[cfg(test)]
mod test {
    use super::{Diff, Node, PatchNode};

    #[test]
    fn same_single() {
        let text1: Node = "a".into();
        let text2 = "a".into();
        assert_eq!(text1.diff(&text2), None);
    }

    #[test]
    fn same_list() {
        let list1: Node = vec!["a".into()].into();
        let list2 = vec!["a".into()].into();
        assert_eq!(list1.diff(&list2), None);
    }

    #[test]
    fn different() {
        let mut list: Node = vec!["a".into()].into();
        let text = "a".into();
        assert_ne!(list, text);
        let patch = list.diff(&text);
        assert_eq!(patch, Some(PatchNode::Replace(text.clone())));
        list.apply(patch.unwrap()).unwrap();
        assert_eq!(list, text);
    }

    #[test]
    fn serde() {
        let node1: Node = "a".into();
        let ser = bincode::serialize(&node1).unwrap();
        let node2 = bincode::deserialize(&ser).unwrap();
        assert_eq!(node1, node2);
    }
}
