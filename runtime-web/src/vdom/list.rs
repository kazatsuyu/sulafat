use super::{ApplyResult, Diff, Node, PatchNode, PatchSingle, Single};
use std::{collections::HashMap, iter::FromIterator, ops::Deref, ops::DerefMut};

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct List {
    list: Vec<Single>,
}

impl List {
    fn key_map(&self) -> HashMap<String, usize> {
        let mut map = HashMap::new();
        for (index, single) in self.iter().enumerate() {
            if let Some(key) = single.key() {
                #[cfg(feature = "nightly-features")]
                map.raw_entry_mut()
                    .from_key(key)
                    .or_insert_with(|| (key.clone(), index));
                #[cfg(not(feature = "nightly-features"))]
                map.entry(key.clone()).or_insert(index);
            }
        }
        map
    }
}

impl Deref for List {
    type Target = Vec<Single>;
    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl DerefMut for List {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.list
    }
}

impl From<List> for Node {
    fn from(list: List) -> Self {
        Node::List(list)
    }
}

enum Reorder {
    Nop,
    New,
    From(usize),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum PatchListNoMoveOp {
    Nop,
    Modify(PatchSingle),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum PatchListOp {
    Nop,
    Modify(PatchSingle),
    From(usize),
    FromModify(usize, PatchSingle),
    New(Single),
}
use serde_derive::{Serialize, Deserialize};

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
    fn reorder<'a>(&'a self, other: &'a List) -> Option<impl Iterator<Item = Reorder> + 'a> {
        let key_map = self.key_map();
        if key_map.is_empty() {
            return None;
        }
        let mut index = 0;
        Some(
            other
                .list
                .iter()
                .enumerate()
                .map(move |(to_index, single)| {
                    if let Some(key) = single.key() {
                        if let Some(&index) = key_map.get(key) {
                            if to_index == index {
                                Reorder::Nop
                            } else {
                                Reorder::From(index)
                            }
                        } else {
                            Reorder::New
                        }
                    } else {
                        if index < self.len() {
                            index += self
                                .list
                                .iter()
                                .skip(index)
                                .position(|single| single.key().is_none())
                                .unwrap_or(self.len() - index);
                            if index < self.len() {
                                let r = if to_index == index {
                                    Reorder::Nop
                                } else {
                                    Reorder::From(index)
                                };
                                index += 1;
                                return r;
                            }
                        }
                        Reorder::New
                    }
                }),
        )
    }
}

impl Diff for List {
    type Patch = PatchList;
    fn diff(&self, other: &Self) -> Option<Self::Patch> {
        let mut nop_count = 0;
        let mut is_move = false;
        let patches = match self.reorder(other) {
            Some(itr) => itr
                .enumerate()
                .map(|(index, reorder)| match reorder {
                    Reorder::New => PatchListOp::New(other[index].clone()),
                    Reorder::Nop => match self[index].diff(&other[index]) {
                        None => {
                            nop_count += 1;
                            PatchListOp::Nop
                        }
                        Some(PatchSingle::Replace(single)) => PatchListOp::New(single),
                        Some(PatchSingle::Element(patch)) => PatchListOp::Modify(patch.into()),
                    },
                    Reorder::From(i) => {
                        is_move = true;
                        match self[i].diff(&other[index]) {
                            None => PatchListOp::From(i),
                            Some(PatchSingle::Replace(single)) => PatchListOp::New(single),
                            Some(PatchSingle::Element(patch)) => {
                                PatchListOp::FromModify(i, patch.into())
                            }
                        }
                    }
                })
                .collect::<Vec<_>>(),
            None => other
                .list
                .iter()
                .enumerate()
                .map(|(index, single)| {
                    if index < self.len() {
                        match self[index].diff(single) {
                            None => {
                                nop_count += 1;
                                PatchListOp::Nop
                            }
                            Some(PatchSingle::Replace(single)) => PatchListOp::New(single),
                            Some(PatchSingle::Element(patch)) => PatchListOp::Modify(patch.into()),
                        }
                    } else {
                        PatchListOp::New(single.clone())
                    }
                })
                .collect(),
        };
        Some(if !is_move && nop_count >= (other.len() + 1) / 2 {
            let len = other.len();
            let entries = patches
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
            PatchList::All(patches)
        })
    }
    fn apply(&mut self, patch: Self::Patch) -> ApplyResult {
        match patch {
            PatchList::All(patch) => {
                let mut old: Vec<_> =
                    std::mem::replace(&mut self.list, Vec::with_capacity(patch.len()))
                        .into_iter()
                        .map(|single| Some(single))
                        .collect();
                for op in patch {
                    match op {
                        PatchListOp::Nop => self
                            .list
                            .push(std::mem::replace(&mut old[self.len()], None).unwrap()),
                        PatchListOp::Modify(patch) => {
                            let mut single = std::mem::replace(&mut old[self.len()], None).unwrap();
                            single.apply(patch)?;
                            self.push(single);
                        }
                        PatchListOp::From(index) => self
                            .list
                            .push(std::mem::replace(&mut old[index], None).unwrap()),
                        PatchListOp::FromModify(index, patch) => {
                            let mut single = std::mem::replace(&mut old[index], None).unwrap();
                            single.apply(patch)?;
                            self.push(single);
                        }
                        PatchListOp::New(single) => {
                            self.push(single);
                        }
                    }
                }
            }
            PatchList::Entries(len, entries) => {
                if len > self.len() {
                    self.list.reserve(len - self.len());
                }
                if len < self.len() {
                    self.truncate(len)
                }
                for (index, patch) in entries {
                    if index >= self.len() {
                        if let PatchSingle::Replace(single) = patch {
                            self.push(single)
                        } else {
                            return Err("変更を行う要素がありません".to_string());
                        }
                    } else {
                        self[index].apply(patch)?;
                    }
                }
            }
            PatchList::Truncate(len) => self.truncate(len),
        }
        Ok(())
    }
}

impl From<Vec<Single>> for List {
    fn from(list: Vec<Single>) -> Self {
        Self { list }
    }
}

impl FromIterator<Single> for List {
    fn from_iter<T: IntoIterator<Item = Single>>(iter: T) -> Self {
        Self {
            list: iter.into_iter().collect(),
        }
    }
}

impl From<Vec<Single>> for Node {
    fn from(list: Vec<Single>) -> Self {
        List::from(list).into()
    }
}

impl FromIterator<Single> for Node {
    fn from_iter<T: IntoIterator<Item = Single>>(iter: T) -> Self {
        List::from_iter(iter).into()
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
            Some(PatchList::All(vec![PatchListOp::New(list2[0].clone())])),
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
                vec![(1, PatchSingle::Replace(list2[1].clone()))]
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
                vec![(0, PatchSingle::Replace(list2[0].clone()))],
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
                PatchListOp::New(list2[1].clone()),
            ])),
        );
        list1.apply(patch.unwrap()).unwrap();
        assert_eq!(list1, list2)
    }
}
