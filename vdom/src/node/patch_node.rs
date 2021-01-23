use crate::{PatchList, PatchSingle, RenderedNode};
use serde_derive::Deserialize;
use sulafat_macros::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PatchNode {
    Replace(RenderedNode),
    Single(PatchSingle),
    List(PatchList),
}
