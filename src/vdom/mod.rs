pub mod diff;
pub mod element;
pub mod list;
pub mod node;
pub mod single;

pub use diff::{ApplyResult, Diff};
pub use element::{Common, Div, Element, PatchCommon, PatchElement, Span};
pub use list::{List, PatchList};
pub use node::{Node, PatchNode};
pub use single::{PatchSingle, Single};
