use crate::{single::RenderedSingle, PatchNode, PatchSingle};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatchListOp {
    Nop,
    Modify(PatchSingle),
    From(usize),
    FromModify(usize, PatchSingle),
    New(RenderedSingle),
}
