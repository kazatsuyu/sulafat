#![cfg_attr(
    feature = "nightly-features",
    feature(unsafe_block_in_unsafe_fn, hash_raw_entry)
)]
#![cfg_attr(feature = "nightly-features", deny(unsafe_op_in_unsafe_fn))]
#![cfg_attr(not(feature = "nightly-features"), allow(unused_unsafe))]

pub mod closure_id;
pub mod diff;
pub mod element;
pub mod list;
pub mod node;
pub mod program;
pub mod single;
pub(crate) mod util;
pub mod view;

pub use closure_id::ClosureId;
pub use diff::{ApplyResult, Diff};
pub use element::{
    id, on_click, on_pointer_move, Common, Div, Element, PatchCommon, PatchElement, Span,
};
pub use list::{List, PatchList};
pub use node::{Node, PatchNode};
pub use program::{EventHandler, Manager, Program};
pub use single::{PatchSingle, Single};
pub use view::CachedView;
