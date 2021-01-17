use serde_derive::{Deserialize, Serialize};

use crate::{
    single::RenderedSingle, Apply, ApplyResult, Div, PatchCommon, PatchElement, PatchNode,
    PatchSingle, RenderedElement, RenderedNode,
};

use super::RenderedCommon;

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "Div")]
pub struct RenderedDiv {
    pub(crate) common: RenderedCommon,
}

impl RenderedDiv {
    pub fn new(common: RenderedCommon) -> Self {
        Self { common }
    }
}

impl From<RenderedDiv> for RenderedElement {
    fn from(div: RenderedDiv) -> Self {
        RenderedElement::Div(div)
    }
}

impl From<RenderedDiv> for RenderedSingle {
    fn from(div: RenderedDiv) -> Self {
        RenderedSingle::Element(div.into())
    }
}

impl From<RenderedDiv> for RenderedNode {
    fn from(div: RenderedDiv) -> Self {
        RenderedNode::Single(div.into())
    }
}

impl<Msg> From<&Div<Msg>> for RenderedDiv {
    fn from(div: &Div<Msg>) -> Self {
        Self {
            common: (&div.common).into(),
        }
    }
}

impl Apply for RenderedDiv {
    type Patch = PatchDiv;
    fn apply(&mut self, patch: Self::Patch) -> ApplyResult {
        self.common.apply(patch.common)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PatchDiv {
    pub(crate) common: PatchCommon,
}

impl From<PatchDiv> for PatchElement {
    fn from(patch: PatchDiv) -> Self {
        PatchElement::Div(patch)
    }
}

impl From<PatchDiv> for PatchSingle {
    fn from(patch: PatchDiv) -> Self {
        PatchElement::from(patch).into()
    }
}

impl From<PatchDiv> for PatchNode {
    fn from(patch: PatchDiv) -> Self {
        PatchSingle::from(patch).into()
    }
}
