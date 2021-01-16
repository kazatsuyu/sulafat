use crate::{Node, PatchList, PatchSingle};
use serde_derive::Deserialize;
use sulafat_macros::Serialize;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum PatchNode<Msg> {
    Replace(Node<Msg>),
    Single(PatchSingle<Msg>),
    List(PatchList<Msg>),
}
