use super::{ApplyResult, Diff, Node, PatchNode, PatchSingle, Single};
use serde::{
    de::{SeqAccess, Visitor},
    ser::SerializeSeq,
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_derive::{Deserialize, Serialize};
use std::{
    cmp::min,
    collections::HashMap,
    fmt::{self, Formatter},
    iter::FromIterator,
    mem::replace,
    ops::Deref,
};

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct List {
    flat_len: usize,
    list: Vec<Node>,
}

impl List {
    fn key_map_indexes(&self) -> (HashMap<&String, usize>, Vec<usize>) {
        let mut map = HashMap::new();
        let mut vec = Vec::with_capacity(self.len());
        let mut i = 0;
        for (index, node) in self.iter().enumerate() {
            if let Some(key) = node.key() {
                #[cfg(feature = "nightly-features")]
                map.raw_entry_mut()
                    .from_key(key)
                    .or_insert_with(|| (key, index));
                #[cfg(not(feature = "nightly-features"))]
                map.entry(key).or_insert(index);
            }
            vec.push(i);
            i += node.flat_len();
        }
        (map, vec)
    }
    pub(super) fn flat_len(&self) -> usize {
        self.iter().map(|node| node.flat_len()).sum()
    }

    pub fn push(&mut self, node: Node) {
        self.flat_len += node.flat_len();
        self.list.push(node);
    }

    pub fn pop(&mut self) -> Option<Node> {
        let node = self.list.pop()?;
        self.flat_len -= node.flat_len();
        Some(node)
    }

    pub fn insert(&mut self, index: usize, node: Node) {
        self.flat_len += node.flat_len();
        self.list.insert(index, node);
    }

    pub fn remove(&mut self, index: usize) -> Node {
        let node = self.list.remove(index);
        self.flat_len -= node.flat_len();
        node
    }

    fn flat_drain_impl(&mut self, buffer: &mut Vec<Option<Single>>) {
        for node in self.list.drain(..) {
            match node {
                Node::List(mut list) => list.flat_drain_impl(buffer),
                Node::Single(single) => buffer.push(Some(single)),
            }
        }
    }

    fn flat_drain(&mut self) -> Vec<Option<Single>> {
        let mut buffer = Vec::with_capacity(self.flat_len);
        self.flat_drain_impl(&mut buffer);
        buffer
    }

    fn flatten_new(&mut self, list: Vec<Node>) {
        for node in list {
            match node {
                Node::List(list) => self.flatten_new(list.list),
                node => self.list.push(node),
            }
        }
    }

    unsafe fn unsafe_flatten(src: Node, mut dest: *mut Node) -> *mut Node {
        if let Node::List(list) = src {
            for node in list.list.into_iter().rev() {
                #[cfg(feature = "nightly-features")]
                {
                    dest = unsafe { Self::unsafe_flatten(node, dest) };
                }
                #[cfg(not(feature = "nightly-features"))]
                {
                    dest = Self::unsafe_flatten(node, dest);
                }
            }
            dest
        } else {
            #[cfg(feature = "nightly-features")]
            {
                unsafe { dest.write(src) };
                unsafe { dest.offset(-1) }
            }
            #[cfg(not(feature = "nightly-features"))]
            {
                dest.write(src);
                dest.offset(-1)
            }
        }
    }

    fn flatten(&mut self) {
        if self.len() == self.flat_len {
            return;
        }
        if self.capacity() < self.flat_len() {
            let list = replace(&mut self.list, Vec::with_capacity(self.flat_len));
            self.flatten_new(list);
        } else {
            let list = replace(&mut self.list, vec![]);
            let len = list.len();
            let capacity = list.capacity();
            let ptr = list.leak().as_mut_ptr();
            let mut p = unsafe { ptr.offset((self.flat_len - 1) as isize) };
            for i in (0..len).rev() {
                let src = unsafe { ptr.offset(i as isize) };
                p = unsafe { Self::unsafe_flatten(src.read(), p) };
                if p == src {
                    break;
                }
            }
            self.list = unsafe { Vec::from_raw_parts(ptr, self.flat_len, capacity) };
        }
    }
}

impl Deref for List {
    type Target = Vec<Node>;
    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl From<List> for Node {
    fn from(list: List) -> Self {
        Node::List(list)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum PatchListOp {
    Nop,
    Modify(PatchSingle),
    From(usize),
    FromModify(usize, PatchSingle),
    New(Single),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum PatchList {
    All(Vec<PatchListOp>),
    Entries(usize, Vec<(usize, PatchSingle)>),
    Truncate(usize),
}

impl From<PatchList> for PatchNode {
    fn from(patch: PatchList) -> Self {
        PatchNode::List(patch)
    }
}

impl List {
    fn add_patch(&self, patches: &mut Vec<PatchListOp>) {
        for node in &self.list {
            match node {
                Node::Single(single) => patches.push(PatchListOp::New(single.clone())),
                Node::List(list) => list.add_patch(patches),
            }
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
    fn flat_diff(&mut self, this: &List, other: &List, this_flat_index: usize) {
        let (key_map, indexes) = this.key_map_indexes();
        let mut this_index = 0;
        for node in &other.list {
            while this.get(this_index).and_then(|node| node.key()).is_some() {
                this_index += 1;
            }
            let this_index = if let Some(key) = node.key() {
                if let Some(&this_index) = key_map.get(key) {
                    this_index
                } else {
                    this.len()
                }
            } else {
                let i = min(this_index + 1, this.len());
                replace(&mut this_index, i)
            };
            if this_index < this.len() {
                match (&this[this_index], node) {
                    (Node::Single(this), Node::Single(other)) => {
                        let this_flat_index = this_flat_index + indexes[this_index];
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
                    (Node::List(this), Node::List(other)) => {
                        self.flat_diff(this, other, this_flat_index + indexes[this_index])
                    }
                    (_, Node::Single(other)) => {
                        self.patches.push(PatchListOp::New(other.clone()));
                        self.flat_index += 1;
                    }
                    (_, Node::List(other)) => {
                        other.add_patch(&mut self.patches);
                        self.flat_index += other.flat_len();
                    }
                }
            } else {
                self.flat_index += node.flat_len();
                match node {
                    Node::Single(other) => {
                        self.patches.push(PatchListOp::New(other.clone()));
                    }
                    Node::List(other) => {
                        other.add_patch(&mut self.patches);
                        self.flat_index += other.flat_len();
                    }
                }
            }
        }
    }
}

impl Diff for List {
    type Patch = PatchList;
    fn diff(&self, other: &Self) -> Option<Self::Patch> {
        let mut context = FlatDiffContext {
            nop_count: 0,
            is_move: false,
            flat_index: 0,
            patches: Vec::with_capacity(other.len()),
        };
        context.flat_diff(self, other, 0);
        Some(
            if !context.is_move && context.nop_count >= (other.flat_len() + 1) / 2 {
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
    fn apply(&mut self, patch: Self::Patch) -> ApplyResult {
        match patch {
            PatchList::All(patches) => {
                let mut prev = self.flat_drain();
                self.list.reserve(patches.len());
                for (index, patch) in patches.into_iter().enumerate() {
                    match patch {
                        PatchListOp::Nop => self.list.push(
                            prev[index]
                                .take()
                                .ok_or_else(|| String::from("元ノードの取得に失敗しました"))?
                                .into(),
                        ),
                        PatchListOp::From(index) => self.list.push(
                            prev[index]
                                .take()
                                .ok_or_else(|| String::from("元ノードの取得に失敗しました"))?
                                .into(),
                        ),
                        PatchListOp::Modify(patch) => {
                            let mut single = prev[index]
                                .take()
                                .ok_or_else(|| String::from("元ノードの取得に失敗しました"))?;
                            single.apply(patch)?;
                            self.list.push(single.into());
                        }
                        PatchListOp::FromModify(index, patch) => {
                            let mut single = prev[index]
                                .take()
                                .ok_or_else(|| String::from("元ノードの取得に失敗しました"))?;
                            single.apply(patch)?;
                            self.list.push(single.into());
                        }
                        PatchListOp::New(single) => self.list.push(single.into()),
                    }
                }
            }
            PatchList::Entries(len, entries) => {
                if len > self.flat_len {
                    self.list.reserve(len - self.len());
                }
                self.flatten();
                self.list.truncate(len);
                for (index, patch) in entries {
                    if index >= self.len() {
                        if let PatchSingle::Replace(single) = patch {
                            self.list.push(single.into())
                        } else {
                            return Err("不正なパッチです".into());
                        }
                    } else if let Node::Single(single) = &mut self.list[index] {
                        single.apply(patch)?;
                    } else {
                        return Err("不正なパッチです".into());
                    }
                }
            }
            PatchList::Truncate(len) => {
                self.flatten();
                self.list.truncate(len);
            }
        }
        self.flat_len = self.list.len();
        Ok(())
    }
}

impl From<Vec<Node>> for List {
    fn from(list: Vec<Node>) -> Self {
        Self {
            flat_len: list.iter().map(|node| node.flat_len()).sum(),
            list,
        }
    }
}

impl FromIterator<Node> for List {
    fn from_iter<T: IntoIterator<Item = Node>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let (min, max) = iter.size_hint();
        let len = max.unwrap_or(min);
        let mut list = Vec::with_capacity(len);
        let mut flat_len = 0;
        for node in iter {
            flat_len += node.flat_len();
            list.push(node);
        }
        Self { flat_len, list }
    }
}

impl From<Vec<Node>> for Node {
    fn from(list: Vec<Node>) -> Self {
        List::from(list).into()
    }
}

impl FromIterator<Node> for Node {
    fn from_iter<T: IntoIterator<Item = Node>>(iter: T) -> Self {
        List::from_iter(iter).into()
    }
}

impl List {
    fn serialize_impl<S>(&self, serialize_seq: &mut S::SerializeSeq) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        for node in &self.list {
            match node {
                Node::Single(single) => serialize_seq.serialize_element(single)?,
                Node::List(list) => list.serialize_impl::<S>(serialize_seq)?,
            }
        }
        Ok(())
    }
}

impl Serialize for List {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut serialize_seq = serializer.serialize_seq(Some(self.flat_len))?;
        self.serialize_impl::<S>(&mut serialize_seq)?;
        serialize_seq.end()
    }
}

struct ListVisitor;

impl<'de> Visitor<'de> for ListVisitor {
    type Value = List;
    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "sequence of Single elements")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut list = Vec::with_capacity(seq.size_hint().unwrap_or(0));
        while let Some(single) = seq.next_element::<Single>()? {
            list.push(single.into());
        }
        Ok(List {
            flat_len: list.len(),
            list,
        })
    }
}

impl<'de> Deserialize<'de> for List {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ListVisitor)
    }
}

#[cfg(test)]
mod test {
    use super::{
        super::{Common, Div, PatchSingle, Span},
        Diff, List, PatchList, PatchListOp,
    };

    #[test]
    fn empty() {
        let list1 = List::default();
        let list2 = List::default();
        assert_eq!(list1.diff(&list2), None)
    }

    #[test]
    fn same() {
        let list1: List = vec![Div::default().into()].into();
        let list2 = vec![Div::default().into()].into();
        assert_eq!(list1, list2);
        assert_eq!(list1.diff(&list2), None);
    }

    #[test]
    fn append() {
        let mut list1 = List::default();
        let list2 = vec![Div::default().into()].into();
        assert_ne!(list1, list2);
        let patch = list1.diff(&list2);
        assert_eq!(
            patch,
            Some(PatchList::All(vec![PatchListOp::New(
                Div::default().into()
            )])),
        );
        list1.apply(patch.unwrap()).unwrap();
        assert_eq!(list1, list2)
    }

    #[test]
    fn remove() {
        let mut list1: List = vec![Div::default().into()].into();
        let list2 = List::default();
        assert_ne!(list1, list2);
        let patch = list1.diff(&list2);
        assert_eq!(patch, Some(PatchList::Truncate(0)),);
        list1.apply(patch.unwrap()).unwrap();
        assert_eq!(list1, list2)
    }

    #[test]
    fn replace() {
        let mut list1: List = vec![
            Div::default().into(),
            Div::default().into(),
            Div::default().into(),
        ]
        .into();
        let list2 = vec![
            Div::default().into(),
            Span::default().into(),
            Div::default().into(),
        ]
        .into();
        assert_ne!(list1, list2);
        let patch = list1.diff(&list2);
        assert_eq!(
            patch,
            Some(PatchList::Entries(
                3,
                vec![(1, PatchSingle::Replace(Span::default().into()))]
            )),
        );
        list1.apply(patch.unwrap()).unwrap();
        assert_eq!(list1, list2)
    }

    #[test]
    fn key_move() {
        let mut list1: List = vec![
            Div::new(Common::new(Some("a".into()), None, vec![].into())).into(),
            Div::default().into(),
        ]
        .into();
        let list2: List = vec![
            Div::default().into(),
            Div::new(Common::new(Some("a".into()), None, vec![].into())).into(),
        ]
        .into();
        assert_ne!(list1, list2);
        let patch = list1.diff(&list2);
        assert_eq!(
            patch,
            Some(PatchList::All(vec![
                PatchListOp::From(1),
                PatchListOp::From(0),
            ])),
        );
        list1.apply(patch.unwrap()).unwrap();
        assert_eq!(list1, list2)
    }

    #[test]
    fn different_key() {
        let mut list1: List = vec![
            Div::new(Common::new(Some("a".into()), None, vec![].into())).into(),
            Div::default().into(),
        ]
        .into();
        let list2: List = vec![
            Div::new(Common::new(Some("b".into()), None, vec![].into())).into(),
            Div::default().into(),
        ]
        .into();
        assert_ne!(list1, list2);
        let patch = list1.diff(&list2);
        assert_eq!(
            patch,
            Some(PatchList::Entries(
                2,
                vec![(
                    0,
                    PatchSingle::Replace(
                        Div::new(Common::new(Some("b".into()), None, vec![].into())).into()
                    )
                )],
            )),
        );
        list1.apply(patch.unwrap()).unwrap();
        assert_eq!(list1, list2)
    }

    #[test]
    fn different_key_move() {
        let mut list1: List = vec![
            Div::new(Common::new(Some("a".into()), None, vec![].into())).into(),
            Div::default().into(),
        ]
        .into();
        let list2: List = vec![
            Div::default().into(),
            Div::new(Common::new(Some("b".into()), None, vec![].into())).into(),
        ]
        .into();
        assert_ne!(list1, list2);
        let patch = list1.diff(&list2);
        assert_eq!(
            patch,
            Some(PatchList::All(vec![
                PatchListOp::From(1),
                PatchListOp::New(
                    Div::new(Common::new(Some("b".into()), None, vec![].into())).into()
                ),
            ])),
        );
        list1.apply(patch.unwrap()).unwrap();
        assert_eq!(list1, list2)
    }
}
