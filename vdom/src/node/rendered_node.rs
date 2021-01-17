use crate::{single::RenderedSingle, Apply, ApplyResult, Node, PatchNode, RenderedList};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "Node")]
pub enum RenderedNode {
    Single(RenderedSingle),
    List(RenderedList),
}

impl<Msg> From<&Node<Msg>> for RenderedNode {
    fn from(mut node: &Node<Msg>) -> Self {
        loop {
            match node {
                Node::Single(single) => {
                    return RenderedNode::Single(single.into());
                }
                Node::List(list) => {
                    return RenderedNode::List(list.into());
                }
                Node::CachedView(cached_view) => {
                    node = unsafe { cached_view.rendered() }.unwrap();
                }
            }
        }
    }
}

impl Apply for RenderedNode {
    type Patch = PatchNode;
    fn apply(&mut self, patch: Self::Patch) -> ApplyResult {
        match patch {
            PatchNode::Replace(node) => *self = node,
            PatchNode::Single(patch) => {
                if let RenderedNode::Single(single) = self {
                    single.apply(patch)?
                } else {
                    return Err("単一ノードではありません。".into());
                }
            }
            PatchNode::List(patch) => {
                if let RenderedNode::List(list) = self {
                    list.apply(patch)?
                } else {
                    return Err("リストではありません。".into());
                }
            }
        }
        Ok(())
    }
}
