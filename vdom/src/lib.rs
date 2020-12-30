#![cfg_attr(
    feature = "nightly-features",
    feature(unsafe_block_in_unsafe_fn, hash_raw_entry)
)]
#![cfg_attr(feature = "nightly-features", deny(unsafe_op_in_unsafe_fn))]

pub mod component;
pub mod diff;
pub mod element;
pub mod list;
pub mod node;
pub mod single;

pub use component::ComponentNode;
pub use diff::{ApplyResult, Diff};
pub use element::{Common, Div, Element, PatchCommon, PatchElement, Span};
pub use list::{List, PatchList};
pub use node::{Node, PatchNode};
pub use single::{PatchSingle, Single};
