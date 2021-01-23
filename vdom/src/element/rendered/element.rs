use serde_derive::{Deserialize, Serialize};

use crate::{
    single::RenderedSingle, Apply, ApplyResult, Element, PatchDiv, PatchNode, PatchSingle,
    PatchSpan,
};

use super::{RenderedDiv, RenderedSpan};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "Element")]
pub enum RenderedElement {
    Div(RenderedDiv),
    Span(RenderedSpan),
}

impl From<RenderedElement> for RenderedSingle {
    fn from(element: RenderedElement) -> Self {
        RenderedSingle::Element(element)
    }
}

impl<Msg> From<&Element<Msg>> for RenderedElement {
    fn from(element: &Element<Msg>) -> Self {
        macro_rules! from_element {
            ($($id:ident),*) => {
                match element {
                    $(Element::$id(element) => {
                        RenderedElement::$id(element.into())
                    })*
                }
            };
        }
        from_element!(Div, Span)
    }
}

impl Apply for RenderedElement {
    type Patch = PatchElement;
    fn apply(&mut self, patch: Self::Patch) -> ApplyResult {
        macro_rules! apply_element {
            ($($id:ident),*) => {
                match patch {
                    PatchElement::Replace(element) => {
                        *self = element;
                        return Ok(());
                    }
                    $(PatchElement::$id(patch) => {
                        if let RenderedElement::$id(element) = self {
                            return element.apply(patch);
                        }
                    })*
                }
            };
        }
        apply_element!(Div, Span);
        Err("異なる要素です".into())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PatchElement {
    Replace(RenderedElement),
    Div(PatchDiv),
    Span(PatchSpan),
}

impl From<PatchElement> for PatchSingle {
    fn from(patch: PatchElement) -> Self {
        match patch {
            PatchElement::Replace(element) => PatchSingle::Replace(element.into()),
            _ => PatchSingle::Element(patch),
        }
    }
}

impl From<PatchElement> for PatchNode {
    fn from(patch: PatchElement) -> Self {
        PatchSingle::from(patch).into()
    }
}
