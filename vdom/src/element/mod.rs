mod attribute;
mod attribute_list;
mod common;
mod div;
mod element;
mod rendered;
mod span;

pub use attribute::{id, on_click, on_pointer_move, style, Attribute, Handler};
pub use attribute_list::AttributeList;
pub use common::Common;
pub use div::Div;
pub use element::Element;
pub use rendered::{
    PatchAttributeList, PatchAttributeListOp, PatchCommon, PatchDiv, PatchElement, PatchSpan,
    RenderedAttribute, RenderedAttributeList, RenderedCommon, RenderedDiv, RenderedElement,
    RenderedSpan,
};
pub use span::Span;
