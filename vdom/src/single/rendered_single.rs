use serde_derive::{Deserialize, Serialize};

use crate::{element::RenderedElement, Apply, ApplyResult, PatchSingle, RenderedNode, Single};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename = "Single")]
pub enum RenderedSingle {
    Text(String),
    Element(RenderedElement),
}

impl From<RenderedSingle> for RenderedNode {
    fn from(single: RenderedSingle) -> Self {
        RenderedNode::Single(single)
    }
}

impl<Msg> From<&Single<Msg>> for RenderedSingle {
    fn from(single: &Single<Msg>) -> Self {
        match single {
            Single::Text(string) => RenderedSingle::Text(string.clone()),
            Single::Element(element) => RenderedSingle::Element(element.into()),
        }
    }
}

impl Apply for RenderedSingle {
    type Patch = PatchSingle;
    fn apply(&mut self, patch: Self::Patch) -> ApplyResult {
        match patch {
            PatchSingle::Replace(node) => *self = node,
            PatchSingle::Element(patch) => {
                if let RenderedSingle::Element(element) = self {
                    element.apply(patch)?
                } else {
                    return Err("Elementではありません".into());
                }
            }
        }
        Ok(())
    }
}
