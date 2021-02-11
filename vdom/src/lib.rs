#![cfg_attr(feature = "nightly-features", feature(unsafe_block_in_unsafe_fn))]
#![cfg_attr(feature = "nightly-features", deny(unsafe_op_in_unsafe_fn))]
#![cfg_attr(not(feature = "nightly-features"), allow(unused_unsafe))]

pub mod attribute;
pub mod closure_id;
pub mod cmd;
pub mod diff;
pub mod element;
pub mod list;
pub mod node;
pub mod program;
pub mod random;
pub mod single;
pub mod timer;
pub(crate) mod util;
pub mod variant_ident;
pub mod view;

pub use attribute::{
    id, on_click, on_pointer_move, style, Attribute, AttributeList, Handler, PatchAttributeList,
    PatchAttributeListOp, RenderedAttribute, RenderedAttributeList, Style,
};
pub use closure_id::ClosureId;
pub use diff::{Apply, ApplyResult, Diff};
pub use element::{
    Common, Div, Element, PatchCommon, PatchDiv, PatchElement, PatchSpan, RenderedElement, Span,
};
pub use list::{List, PatchList, PatchListOp, RenderedList};
pub use node::{Node, PatchNode, RenderedNode};
pub use program::{EventHandler, Manager, Program};
pub use single::{PatchSingle, Single};
pub use variant_ident::VariantIdent;
pub use view::CachedView;

extern crate self as sulafat_vdom;
