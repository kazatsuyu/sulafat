use super::{
    ApplyResult, Common, Diff, Element, Node, PatchCommon, PatchElement, PatchNode, PatchSingle,
    Single,
};
use serde_derive::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Div {
    pub(crate) common: Common,
}

impl Div {
    pub fn new(common: Common) -> Self {
        Self { common }
    }

    pub fn common(&self) -> &Common {
        &self.common
    }

    pub fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }
}

impl From<Div> for Element {
    fn from(div: Div) -> Self {
        Element::Div(div)
    }
}

impl From<Div> for Single {
    fn from(div: Div) -> Self {
        Element::from(div).into()
    }
}

impl From<Div> for Node {
    fn from(div: Div) -> Self {
        Single::from(div).into()
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

impl Diff for Div {
    type Patch = PatchDiv;
    fn diff(&self, other: &Self) -> Option<Self::Patch> {
        Some(PatchDiv {
            common: self.common.diff(&other.common)?,
        })
    }
    fn apply(&mut self, patch: Self::Patch) -> ApplyResult {
        self.common.apply(patch.common)
    }
}

#[cfg(test)]
mod test {
    use super::{Common, Diff, Div, PatchCommon, PatchDiv};
    #[test]
    fn same() {
        let div1 = Div::default();
        let div2 = Div::default();
        assert_eq!(div1.diff(&div2), None)
    }

    #[test]
    fn different_id() {
        let mut div1 = Div::new(Common::new(None, Some("a".into()), Default::default()));
        let div2 = Div::new(Common::new(None, Some("b".into()), Default::default()));
        assert_ne!(div1, div2);
        let patch = div1.diff(&div2);
        assert_eq!(
            patch,
            Some(PatchDiv {
                common: PatchCommon {
                    id: Some(Some("b".into())),
                    children: Default::default()
                }
            })
        );
        div1.apply(patch.unwrap()).unwrap();
        assert_eq!(div1, div2);
    }
}
