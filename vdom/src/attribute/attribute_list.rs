use std::{cmp::Ordering, iter::FromIterator, ops::Deref};

use crate::{Attribute, Diff, PatchAttributeList, PatchAttributeListOp, VariantIdent};
use sulafat_macros::{Clone, PartialEq, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AttributeList<Msg> {
    list: Vec<Attribute<Msg>>,
}

impl<Msg> AttributeList<Msg> {
    pub(crate) fn new(mut list: Vec<Attribute<Msg>>) -> Self {
        list.sort_by(|a, b| a.variant_ident().cmp(&b.variant_ident()));
        list.dedup_by(|a, b| a.variant_ident() == b.variant_ident());
        Self { list }
    }
}

impl<Msg> Deref for AttributeList<Msg> {
    type Target = Vec<Attribute<Msg>>;
    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl<Msg> Default for AttributeList<Msg> {
    fn default() -> Self {
        Self {
            list: Default::default(),
        }
    }
}

impl<Msg> From<Vec<Attribute<Msg>>> for AttributeList<Msg> {
    fn from(list: Vec<Attribute<Msg>>) -> Self {
        Self::new(list)
    }
}

impl<Msg> FromIterator<Attribute<Msg>> for AttributeList<Msg> {
    fn from_iter<T: IntoIterator<Item = Attribute<Msg>>>(iter: T) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl<Msg> Diff for AttributeList<Msg> {
    type Patch = PatchAttributeList;
    fn diff(&self, other: &mut Self) -> Option<Self::Patch> {
        let mut i1 = 0;
        let mut i2 = 0;
        let mut list = vec![];
        while i1 < self.len() && i2 < other.len() {
            let this = &self[i1];
            let other = &other[i2];
            match this.variant_ident().cmp(&other.variant_ident()) {
                Ordering::Less => {
                    i1 += 1;
                    list.push(PatchAttributeListOp::Remove(this.variant_ident().into()))
                }
                Ordering::Greater => {
                    i2 += 1;
                    list.push(PatchAttributeListOp::Insert(other.into()))
                }
                Ordering::Equal => {
                    i1 += 1;
                    i2 += 1;
                    if this != other {
                        list.push(PatchAttributeListOp::Insert(other.into()))
                    }
                }
            }
        }
        while i1 < self.len() {
            list.push(PatchAttributeListOp::Remove(
                self[i1].variant_ident().into(),
            ));
            i1 += 1;
        }
        while i2 < other.len() {
            list.push(PatchAttributeListOp::Insert((&other[i2]).into()));
            i2 += 1;
        }
        if list.is_empty() {
            None
        } else {
            Some(list.into())
        }
    }
}
