use std::fmt;

use super::{ApplyResult, ComponentNode, Diff, List, PatchList, PatchSingle, Single};
use fmt::Formatter;
use serde::{
    de::{EnumAccess, VariantAccess, Visitor},
    ser::SerializeTupleVariant,
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug)]
pub enum Node<Msg> {
    Single(Single<Msg>),
    List(List<Msg>),
    Component(ComponentNode<Msg>),
}

impl<Msg> Node<Msg> {
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
pub enum PatchNode<Msg> {
    Replace(Node<Msg>),
    Single(PatchSingle<Msg>),
    List(PatchList<Msg>),
}

impl<Msg> Diff for Node<Msg> {
    type Patch = PatchNode<Msg>;
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

impl<Msg> Serialize for Node<Msg> {
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

impl<'de, Msg> Deserialize<'de> for Node<Msg> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct NodeVisitor<Msg>(std::marker::PhantomData<Msg>);

        impl<'de, Msg> Visitor<'de> for NodeVisitor<Msg> {
            type Value = Node<Msg>;
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
                    VariantTag::Single => variant.newtype_variant::<Single<Msg>>()?.into(),
                    VariantTag::List => variant.newtype_variant::<List<Msg>>()?.into(),
                })
            }
        }
        deserializer.deserialize_enum("Node", &["Single", "List"], NodeVisitor(Default::default()))
    }
}

impl<Msg> Clone for Node<Msg> {
    fn clone(&self) -> Self {
        match self {
            Node::Single(single) => Node::Single(single.clone()),
            Node::List(list) => Node::List(list.clone()),
            Node::Component(component) => Node::Component(component.clone()),
        }
    }
}

impl<Msg> PartialEq for Node<Msg> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Node::Single(this), Node::Single(other)) => this == other,
            (Node::List(this), Node::List(other)) => this == other,
            (Node::Component(this), Node::Component(other)) => this == other,
            _ => false,
        }
    }
}

impl<Msg> Eq for Node<Msg> {}

#[cfg(test)]
mod test {
    use super::{Diff, Node, PatchNode};

    #[test]
    fn same_single() {
        let text1: Node<()> = "a".into();
        let text2 = "a".into();
        assert_eq!(text1.diff(&text2), None);
    }

    #[test]
    fn same_list() {
        let list1: Node<()> = vec!["a".into()].into();
        let list2 = vec!["a".into()].into();
        assert_eq!(list1.diff(&list2), None);
    }

    #[test]
    fn different() {
        let mut list: Node<()> = vec!["a".into()].into();
        let text = "a".into();
        assert_ne!(list, text);
        let patch = list.diff(&text);
        assert_eq!(patch, Some(PatchNode::Replace(text.clone())));
        list.apply(patch.unwrap()).unwrap();
        assert_eq!(list, text);
    }

    #[test]
    fn serde() {
        let node1: Node<()> = "a".into();
        let ser = bincode::serialize(&node1).unwrap();
        let node2 = bincode::deserialize(&ser).unwrap();
        assert_eq!(node1, node2);
    }
}
