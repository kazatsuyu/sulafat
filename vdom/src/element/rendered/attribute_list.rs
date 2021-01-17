use std::ops::Deref;

use crate::{element::AttributeList, Apply, ApplyResult, VariantIdent};
use serde_derive::{Deserialize, Serialize};

use super::{attribute, RenderedAttribute};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "AttributeList")]
pub struct RenderedAttributeList {
    list: Vec<RenderedAttribute>,
}

impl From<Vec<RenderedAttribute>> for RenderedAttributeList {
    fn from(mut list: Vec<RenderedAttribute>) -> Self {
        list.sort_by(|a, b| a.variant_ident().cmp(&b.variant_ident()));
        list.dedup_by(|a, b| a.variant_ident() == b.variant_ident());
        Self { list }
    }
}

impl Deref for RenderedAttributeList {
    type Target = Vec<RenderedAttribute>;
    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl<Msg> From<&AttributeList<Msg>> for RenderedAttributeList {
    fn from(list: &AttributeList<Msg>) -> Self {
        Self {
            list: list.iter().map(|attribute| attribute.into()).collect(),
        }
    }
}

impl Apply for RenderedAttributeList {
    type Patch = PatchAttributeList;
    fn apply(&mut self, patch: Self::Patch) -> ApplyResult {
        if patch.list.is_empty() {
            return Ok(());
        }
        let mut i = 0;
        for patch in patch {
            match patch {
                PatchAttributeListOp::Remove(r#type) => {
                    while i < self.len() && self[i].variant_ident() < r#type {
                        i += 1;
                    }
                    if let Some(attribute) = self.get(i) {
                        if attribute.variant_ident() != r#type {
                            return Err("削除する属性がありません".into());
                        }
                    } else {
                        return Err("削除する属性がありません".into());
                    }
                    self.list.remove(i);
                }
                PatchAttributeListOp::Insert(attribute) => {
                    while i < self.len() && self[i].variant_ident() < attribute.variant_ident() {
                        i += 1;
                    }
                    if i < self.len() && self[i].variant_ident() == attribute.variant_ident() {
                        self.list[i] = attribute;
                    } else {
                        self.list.insert(i, attribute);
                    }
                    i += 1;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatchAttributeList {
    list: Vec<PatchAttributeListOp>,
}

impl From<Vec<PatchAttributeListOp>> for PatchAttributeList {
    fn from(list: Vec<PatchAttributeListOp>) -> Self {
        Self { list }
    }
}

impl IntoIterator for PatchAttributeList {
    type Item = PatchAttributeListOp;
    type IntoIter = <Vec<PatchAttributeListOp> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.list.into_iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatchAttributeListOp {
    Remove(<RenderedAttribute as VariantIdent>::Type),
    Insert(RenderedAttribute),
}
