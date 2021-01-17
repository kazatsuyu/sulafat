use serde_derive::{Deserialize, Serialize};

use crate::{
    single::RenderedSingle, Apply, ApplyResult, PatchCommon, PatchElement, PatchNode, PatchSingle,
    RenderedElement, RenderedNode, Span,
};

use super::RenderedCommon;

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "Span")]
pub struct RenderedSpan {
    pub(crate) common: RenderedCommon,
}

impl RenderedSpan {
    pub fn new(common: RenderedCommon) -> Self {
        Self { common }
    }
}

impl From<RenderedSpan> for RenderedElement {
    fn from(span: RenderedSpan) -> Self {
        RenderedElement::Span(span)
    }
}

impl From<RenderedSpan> for RenderedSingle {
    fn from(span: RenderedSpan) -> Self {
        RenderedSingle::Element(span.into())
    }
}

impl From<RenderedSpan> for RenderedNode {
    fn from(span: RenderedSpan) -> Self {
        RenderedNode::Single(span.into())
    }
}

impl<Msg> From<&Span<Msg>> for RenderedSpan {
    fn from(span: &Span<Msg>) -> Self {
        Self {
            common: (&span.common).into(),
        }
    }
}

impl Apply for RenderedSpan {
    type Patch = PatchSpan;
    fn apply(&mut self, patch: Self::Patch) -> ApplyResult {
        self.common.apply(patch.common)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PatchSpan {
    pub(crate) common: PatchCommon,
}

impl From<PatchSpan> for PatchElement {
    fn from(patch: PatchSpan) -> Self {
        PatchElement::Span(patch)
    }
}

impl From<PatchSpan> for PatchSingle {
    fn from(patch: PatchSpan) -> Self {
        PatchElement::from(patch).into()
    }
}

impl From<PatchSpan> for PatchNode {
    fn from(patch: PatchSpan) -> Self {
        PatchSingle::from(patch).into()
    }
}
