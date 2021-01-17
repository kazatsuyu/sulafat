use std::{any::Any, collections::HashMap, rc::Weak};

use crate::{list::PatchListOp, CachedView, ClosureId, Diff, List, PatchNode, Single};
use serde::{ser::SerializeTupleVariant, Serialize, Serializer};

#[derive(Debug)]
pub enum Node<Msg> {
    Single(Single<Msg>),
    List(List<Msg>),
    CachedView(CachedView<Msg>),
}

impl<Msg> Node<Msg> {
    pub fn key(&self) -> Option<&String> {
        match self {
            Node::Single(single) => single.key(),
            Node::CachedView(view) => view.key(),
            Node::List(_) => None,
        }
    }

    pub(crate) fn flat_len(&self) -> Option<usize> {
        match self {
            Node::Single(_) => Some(1),
            Node::List(list) => list.flat_len(),
            Node::CachedView(view) => view.flat_len(),
        }
    }

    pub(crate) fn is_full_rendered(&self) -> bool {
        match self {
            Node::Single(single) => single.is_full_rendered(),
            Node::List(list) => list.is_full_rendered(),
            Node::CachedView(view) => view.is_full_rendered(),
        }
    }

    pub(crate) fn full_render(&mut self) {
        if self.is_full_rendered() {
            return;
        }
        match self {
            Node::Single(single) => single.full_render(),
            Node::List(list) => list.full_render(),
            Node::CachedView(view) => {
                view.full_render();
            }
        }
    }

    pub(crate) fn add_patch(&mut self, patches: &mut Vec<PatchListOp>) {
        match self {
            Node::Single(single) => patches.push(PatchListOp::New((&*single).into())),
            Node::List(list) => list.add_patch(patches),
            Node::CachedView(view) => view.add_patch(patches),
        }
    }

    pub(crate) fn pick_handler(&self, handlers: &mut HashMap<ClosureId, Weak<dyn Any>>)
    where
        Msg: 'static,
    {
        match self {
            Node::Single(single) => single.pick_handler(handlers),
            Node::List(list) => list.pick_handler(handlers),
            Node::CachedView(view) => unsafe { view.rendered() }.unwrap().pick_handler(handlers),
        }
    }
}

impl<Msg> Diff for Node<Msg> {
    type Patch = PatchNode;
    fn diff(&self, other: &mut Self) -> Option<Self::Patch> {
        match (self, other) {
            (Node::Single(s), Node::Single(o)) => Some(s.diff(o)?.into()),
            (Node::List(s), Node::List(o)) => Some(s.diff(o)?.into()),
            (Node::CachedView(s), Node::CachedView(o)) => s.diff(o),
            (_, other) => Some(PatchNode::Replace((&*other).into())),
        }
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
            Node::CachedView(view) => unsafe { view.rendered() }.unwrap().serialize(serializer),
        }
    }
}

impl<Msg> Clone for Node<Msg> {
    fn clone(&self) -> Self {
        match self {
            Node::Single(single) => Node::Single(single.clone()),
            Node::List(list) => Node::List(list.clone()),
            Node::CachedView(component) => Node::CachedView(component.clone()),
        }
    }
}

impl<Msg> PartialEq for Node<Msg> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Node::Single(this), Node::Single(other)) => this == other,
            (Node::List(this), Node::List(other)) => this == other,
            (Node::CachedView(this), Node::CachedView(other)) => this == other,
            _ => false,
        }
    }
}

impl<Msg> Eq for Node<Msg> {}

#[cfg(test)]
mod test {
    use crate::{Apply, Diff, Node, PatchNode, RenderedNode};

    #[test]
    fn same_single() {
        let text1: Node<()> = "a".into();
        let mut text2 = "a".into();
        assert_eq!(text1.diff(&mut text2), None);
    }

    #[test]
    fn same_list() {
        let list1: Node<()> = vec!["a".into()].into();
        let mut list2 = vec!["a".into()].into();
        assert_eq!(list1.diff(&mut list2), None);
    }

    #[test]
    fn different() {
        let list: Node<()> = vec!["a".into()].into();
        let mut text = "a".into();
        assert_ne!(list, text);
        let patch = list.diff(&mut text);
        assert_eq!(patch, Some(PatchNode::Replace((&text).into())));
        let mut rendered_list = RenderedNode::from(&list);
        let rendered_text = RenderedNode::from(&text);
        rendered_list.apply(patch.unwrap()).unwrap();
        assert_eq!(rendered_list, rendered_text);
    }

    #[test]
    fn serde() {
        let node1: Node<()> = "a".into();
        let ser = bincode::serialize(&node1).unwrap();
        let rendered_node1 = RenderedNode::from(&node1);
        let rendered_node2 = bincode::deserialize(&ser).unwrap();
        assert_eq!(rendered_node1, rendered_node2);
    }
}
