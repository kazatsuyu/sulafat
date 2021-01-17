use crate::{PatchElement, PatchNode};
use serde_derive::{Deserialize, Serialize};

use super::RenderedSingle;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum PatchSingle {
    Replace(RenderedSingle),
    Element(PatchElement),
}

impl From<PatchSingle> for PatchNode {
    fn from(patch: PatchSingle) -> Self {
        match patch {
            PatchSingle::Replace(single) => PatchNode::Replace(single.into()),
            _ => PatchNode::Single(patch),
        }
    }
}
