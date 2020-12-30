use super::{ApplyResult, Diff, List, PatchList, PatchSingle, Single};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Node {
    Single(Single),
    List(List),
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
}
