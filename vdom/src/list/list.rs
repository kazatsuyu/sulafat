use crate::{CachedView, ClosureId, Diff, Node, PatchList, PatchListOp, PatchSingle};
use serde::{ser::SerializeSeq, Serialize, Serializer};
use std::{
    any::Any, cmp::min, collections::HashMap, iter::FromIterator, mem::replace, ops::Deref,
    rc::Weak,
};
use sulafat_macros::{Clone, PartialEq};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct List<Msg> {
    flat_len: usize,
    list: Vec<Node<Msg>>,
    full_rendered_count: usize,
}

impl<Msg> List<Msg> {
    fn key_map_indexes(&self) -> (HashMap<&str, usize>, Vec<usize>) {
        let mut map = HashMap::new();
        let mut vec = Vec::with_capacity(self.len());
        let mut i = 0;
        for (index, node) in self.iter().enumerate() {
            if let Some(key) = node.key() {
                map.entry(key.as_str()).or_insert(index);
            }
            vec.push(i);
            i += node.flat_len().unwrap();
        }
        (map, vec)
    }
    pub(crate) fn flat_len(&self) -> Option<usize> {
        if self.is_full_rendered() {
            Some(self.flat_len)
        } else {
            None
        }
    }

    pub fn push(&mut self, node: Node<Msg>) {
        if node.is_full_rendered() {
            self.full_rendered_count += 1;
            self.flat_len += node.flat_len().unwrap();
        }
        self.list.push(node);
    }

    pub fn pop(&mut self) -> Option<Node<Msg>> {
        let node = self.list.pop()?;
        if node.is_full_rendered() {
            self.full_rendered_count -= 1;
            self.flat_len -= node.flat_len().unwrap();
        }
        Some(node)
    }

    pub fn insert(&mut self, index: usize, node: Node<Msg>) {
        if node.is_full_rendered() {
            self.full_rendered_count += 1;
            self.flat_len += node.flat_len().unwrap();
        }
        self.list.insert(index, node);
    }

    pub fn remove(&mut self, index: usize) -> Node<Msg> {
        let node = self.list.remove(index);
        if node.is_full_rendered() {
            self.full_rendered_count -= 1;
        }
        self.flat_len -= node.flat_len().unwrap();
        node
    }

    pub(crate) fn is_full_rendered(&self) -> bool {
        self.full_rendered_count == self.len()
    }

    pub(crate) fn full_render(&mut self) {
        for node in &mut self.list {
            if !node.is_full_rendered() {
                node.full_render();
                self.full_rendered_count += 1;
            }
        }
    }

    pub(crate) fn pick_handler(&self, handlers: &mut HashMap<ClosureId, Weak<dyn Any>>)
    where
        Msg: 'static,
    {
        for node in &self.list {
            node.pick_handler(handlers)
        }
    }
}

impl<Msg> Deref for List<Msg> {
    type Target = Vec<Node<Msg>>;
    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl<Msg> From<List<Msg>> for Node<Msg> {
    fn from(list: List<Msg>) -> Self {
        Node::List(list)
    }
}

impl<Msg> List<Msg> {
    pub(crate) fn add_patch(&mut self, patches: &mut Vec<PatchListOp>) {
        for node in &mut self.list {
            node.add_patch(patches);
        }
    }
}

struct FlatDiffContext {
    nop_count: usize,
    is_move: bool,
    flat_index: usize,
    patches: Vec<PatchListOp>,
}

impl FlatDiffContext {
    fn node_diff<Msg>(&mut self, this: &Node<Msg>, other: &mut Node<Msg>, this_flat_index: usize) {
        match (&this, other) {
            (Node::Single(this), Node::Single(other)) => {
                let patch = match (this.diff(other), this_flat_index == self.flat_index) {
                    (Some(PatchSingle::Replace(single)), _) => PatchListOp::New(single),
                    (Some(patch), false) => {
                        self.is_move = true;
                        PatchListOp::FromModify(this_flat_index, patch)
                    }
                    (Some(patch), true) => PatchListOp::Modify(patch),
                    (None, false) => {
                        self.is_move = true;
                        PatchListOp::From(this_flat_index)
                    }
                    (None, true) => {
                        self.nop_count += 1;
                        PatchListOp::Nop
                    }
                };
                self.patches.push(patch);
                self.flat_index += 1;
            }
            (Node::List(this), Node::List(other)) => self.flat_diff(this, other, this_flat_index),
            (Node::CachedView(this), Node::CachedView(other)) => {
                self.cached_view_flat_diff(this, other, this_flat_index)
            }
            (_, node) => {
                node.add_patch(&mut self.patches);
                self.flat_index += node.flat_len().unwrap();
            }
        }
    }
    fn flat_diff<Msg>(&mut self, this: &List<Msg>, other: &mut List<Msg>, this_flat_index: usize) {
        let (key_map, indexes) = this.key_map_indexes();
        let mut this_index = 0;
        for node in &mut other.list {
            while this.get(this_index).and_then(|node| node.key()).is_some() {
                this_index += 1;
            }
            let this_index = if let Some(key) = node.key() {
                if let Some(&this_index) = key_map.get(key.as_str()) {
                    this_index
                } else {
                    this.len()
                }
            } else {
                let i = min(this_index + 1, this.len());
                replace(&mut this_index, i)
            };
            if this_index < this.len() {
                self.node_diff(
                    &this.list[this_index],
                    node,
                    this_flat_index + indexes[this_index],
                )
            } else {
                node.add_patch(&mut self.patches);
                self.flat_index += node.flat_len().unwrap();
            }
        }
    }

    fn cached_view_flat_diff<Msg>(
        &mut self,
        this: &CachedView<Msg>,
        other: &mut CachedView<Msg>,
        this_flat_index: usize,
    ) {
        if this.is_different(other) {
            other.render().add_patch(&mut self.patches);
        } else if !this.share_cache_if_same(other) {
            self.node_diff(
                unsafe { this.rendered() }.unwrap(),
                other.render(),
                this_flat_index,
            )
        }
        self.flat_index += other.flat_len().unwrap();
    }
}

impl<Msg> Diff for List<Msg> {
    type Patch = PatchList;
    fn diff(&self, other: &mut Self) -> Option<Self::Patch> {
        let mut context = FlatDiffContext {
            nop_count: 0,
            is_move: false,
            flat_index: 0,
            patches: Vec::with_capacity(other.len()),
        };
        context.flat_diff(self, other, 0);
        Some(
            if !context.is_move && context.nop_count >= (other.flat_len().unwrap() + 1) / 2 {
                let len = other.len();
                let entries = context
                    .patches
                    .into_iter()
                    .enumerate()
                    .filter_map(|(index, op)| match op {
                        PatchListOp::Nop => None,
                        PatchListOp::Modify(patch) => Some((index, patch)),
                        PatchListOp::From(_) => unreachable!(),
                        PatchListOp::FromModify(_, _) => unreachable!(),
                        PatchListOp::New(single) => Some((index, PatchSingle::Replace(single))),
                    })
                    .collect::<Vec<_>>();
                if entries.is_empty() {
                    if len < self.len() {
                        PatchList::Truncate(len)
                    } else {
                        return None;
                    }
                } else {
                    PatchList::Entries(len, entries)
                }
            } else {
                PatchList::All(context.patches)
            },
        )
    }
}

impl<Msg> From<Vec<Node<Msg>>> for List<Msg> {
    fn from(list: Vec<Node<Msg>>) -> Self {
        let (flat_len, full_rendered_count) = list.iter().fold((0, 0), |(a, b), node| {
            (
                a + node.flat_len().unwrap(),
                b + node.is_full_rendered() as usize,
            )
        });
        Self {
            flat_len,
            list,
            full_rendered_count: full_rendered_count.into(),
        }
    }
}

impl<Msg> FromIterator<Node<Msg>> for List<Msg> {
    fn from_iter<T: IntoIterator<Item = Node<Msg>>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let (min, max) = iter.size_hint();
        let len = max.unwrap_or(min);
        let mut list = Vec::with_capacity(len);
        let mut flat_len = 0;
        let mut full_rendered_count = 0;
        for node in iter {
            if node.is_full_rendered() {
                full_rendered_count += 1;
                flat_len += node.flat_len().unwrap();
            }
            list.push(node);
        }
        Self {
            flat_len,
            list,
            full_rendered_count: full_rendered_count,
        }
    }
}

impl<Msg> From<Vec<Node<Msg>>> for Node<Msg> {
    fn from(list: Vec<Node<Msg>>) -> Self {
        List::from(list).into()
    }
}

impl<Msg> FromIterator<Node<Msg>> for Node<Msg> {
    fn from_iter<T: IntoIterator<Item = Node<Msg>>>(iter: T) -> Self {
        List::from_iter(iter).into()
    }
}

impl<Msg> List<Msg> {
    fn serialize_impl<S>(&self, serialize_seq: &mut S::SerializeSeq) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        for node in &self.list {
            Self::serialize_node::<S>(node, serialize_seq)?;
        }
        Ok(())
    }
    fn serialize_node<S>(
        node: &Node<Msg>,
        serialize_seq: &mut S::SerializeSeq,
    ) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        match node {
            Node::Single(single) => serialize_seq.serialize_element(single)?,
            Node::List(list) => list.serialize_impl::<S>(serialize_seq)?,
            Node::CachedView(view) => {
                Self::serialize_node::<S>(unsafe { view.rendered() }.unwrap(), serialize_seq)?;
            }
        }
        Ok(())
    }
}

impl<Msg> Serialize for List<Msg> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut serialize_seq = serializer.serialize_seq(Some(self.flat_len))?;
        self.serialize_impl::<S>(&mut serialize_seq)?;
        serialize_seq.end()
    }
}

impl<Msg> Eq for List<Msg> {}
#[cfg(test)]
mod test {
    use crate::{
        element::{RenderedCommon, RenderedDiv, RenderedSpan},
        Apply, Common, Diff, Div, List, PatchList, PatchListOp, PatchSingle, RenderedList, Span,
    };

    #[test]
    fn empty() {
        let list1 = List::<()>::default();
        let mut list2 = List::default();
        assert_eq!(list1.diff(&mut list2), None)
    }

    #[test]
    fn same() {
        let list1: List<()> = vec![Div::default().into()].into();
        let mut list2 = vec![Div::default().into()].into();
        assert_eq!(list1, list2);
        assert_eq!(list1.diff(&mut list2), None);
    }

    #[test]
    fn append() {
        let list1 = List::<()>::default();
        let mut list2 = vec![Div::default().into()].into();
        assert_ne!(list1, list2);
        let patch = list1.diff(&mut list2);
        assert_eq!(
            patch,
            Some(PatchList::All(vec![PatchListOp::New(
                RenderedDiv::default().into()
            )])),
        );
        let mut rendered_list1 = RenderedList::from(&list1);
        let rendered_list2 = RenderedList::from(&list2);
        rendered_list1.apply(patch.unwrap()).unwrap();
        assert_eq!(rendered_list1, rendered_list2)
    }

    #[test]
    fn remove() {
        let list1: List<()> = vec![Div::default().into()].into();
        let mut list2 = List::default();
        assert_ne!(list1, list2);
        let patch = list1.diff(&mut list2);
        assert_eq!(patch, Some(PatchList::Truncate(0)),);
        let mut rendered_list1 = RenderedList::from(&list1);
        let rendered_list2 = RenderedList::from(&list2);
        rendered_list1.apply(patch.unwrap()).unwrap();
        assert_eq!(rendered_list1, rendered_list2)
    }

    #[test]
    fn replace() {
        let list1: List<()> = vec![
            Div::default().into(),
            Div::default().into(),
            Div::default().into(),
        ]
        .into();
        let mut list2 = vec![
            Div::default().into(),
            Span::default().into(),
            Div::default().into(),
        ]
        .into();
        assert_ne!(list1, list2);
        let patch = list1.diff(&mut list2);
        assert_eq!(
            patch,
            Some(PatchList::Entries(
                3,
                vec![(1, PatchSingle::Replace(RenderedSpan::default().into()))]
            )),
        );
        let mut rendered_list1 = RenderedList::from(&list1);
        let rendered_list2 = RenderedList::from(&list2);
        rendered_list1.apply(patch.unwrap()).unwrap();
        assert_eq!(rendered_list1, rendered_list2)
    }

    #[test]
    fn key_move() {
        let list1: List<()> = vec![
            Div::new(Common::new(Some("a".into()), vec![].into(), vec![].into())).into(),
            Div::default().into(),
        ]
        .into();
        let mut list2: List<()> = vec![
            Div::default().into(),
            Div::new(Common::new(Some("a".into()), vec![].into(), vec![].into())).into(),
        ]
        .into();
        assert_ne!(list1, list2);
        let patch = list1.diff(&mut list2);
        assert_eq!(
            patch,
            Some(PatchList::All(vec![
                PatchListOp::From(1),
                PatchListOp::From(0),
            ])),
        );
        let mut rendered_list1 = RenderedList::from(&list1);
        let rendered_list2 = RenderedList::from(&list2);
        rendered_list1.apply(patch.unwrap()).unwrap();
        assert_eq!(rendered_list1, rendered_list2)
    }

    #[test]
    fn different_key() {
        let list1: List<()> = vec![
            Div::new(Common::new(Some("a".into()), vec![].into(), vec![].into())).into(),
            Div::default().into(),
        ]
        .into();
        let mut list2: List<()> = vec![
            Div::new(Common::new(Some("b".into()), vec![].into(), vec![].into())).into(),
            Div::default().into(),
        ]
        .into();
        assert_ne!(list1, list2);
        let patch = list1.diff(&mut list2);
        assert_eq!(
            patch,
            Some(PatchList::Entries(
                2,
                vec![(
                    0,
                    PatchSingle::Replace(
                        RenderedDiv::new(RenderedCommon::new(vec![].into(), vec![].into())).into()
                    )
                )],
            )),
        );
        let mut rendered_list1 = RenderedList::from(&list1);
        let rendered_list2 = RenderedList::from(&list2);
        rendered_list1.apply(patch.unwrap()).unwrap();
        assert_eq!(rendered_list1, rendered_list2)
    }

    #[test]
    fn different_key_move() {
        let list1: List<()> = vec![
            Div::new(Common::new(Some("a".into()), vec![].into(), vec![].into())).into(),
            Div::default().into(),
        ]
        .into();
        let mut list2: List<()> = vec![
            Div::default().into(),
            Div::new(Common::new(Some("b".into()), vec![].into(), vec![].into())).into(),
        ]
        .into();
        assert_ne!(list1, list2);
        let patch = list1.diff(&mut list2);
        assert_eq!(
            patch,
            Some(PatchList::All(vec![
                PatchListOp::From(1),
                PatchListOp::New(
                    RenderedDiv::new(RenderedCommon::new(vec![].into(), vec![].into())).into()
                ),
            ])),
        );
        let mut rendered_list1 = RenderedList::from(&list1);
        let rendered_list2 = RenderedList::from(&list2);
        rendered_list1.apply(patch.unwrap()).unwrap();
        assert_eq!(rendered_list1, rendered_list2)
    }
}
