use crate::{
    Apply, ApplyResult, Common, PatchAttributeList, PatchList, RenderedAttributeList, RenderedList,
};
use serde_derive::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "Common")]
pub struct RenderedCommon {
    pub(crate) attribute_list: RenderedAttributeList,
    pub(crate) children: RenderedList,
}

impl<Msg> From<&Common<Msg>> for RenderedCommon {
    fn from(common: &Common<Msg>) -> Self {
        Self {
            attribute_list: (&common.attribute_list).into(),
            children: (&common.children).into(),
        }
    }
}

impl RenderedCommon {
    pub fn new(attribute_list: RenderedAttributeList, children: RenderedList) -> Self {
        Self {
            attribute_list,
            children,
        }
    }
}

impl Apply for RenderedCommon {
    type Patch = PatchCommon;
    fn apply(&mut self, patch: Self::Patch) -> ApplyResult {
        self.attribute_list.apply(patch.attribute_list)?;
        if let Some(patch) = patch.children {
            self.children.apply(patch)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatchCommon {
    pub(crate) attribute_list: PatchAttributeList,
    pub(crate) children: Option<PatchList>,
}
