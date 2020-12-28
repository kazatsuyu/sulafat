use super::{
    ApplyResult, Common, Diff, Element, Node, PatchCommon, PatchElement, PatchNode, PatchSingle,
    Single,
};
use serde_derive::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Span {
    pub(crate) common: Common,
}

impl Span {
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

impl From<Span> for Element {
    fn from(span: Span) -> Self {
        Element::Span(span)
    }
}

impl From<Span> for Single {
    fn from(span: Span) -> Self {
        Element::from(span).into()
    }
}

impl From<Span> for Node {
    fn from(span: Span) -> Self {
        Single::from(span).into()
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

impl Diff for Span {
    type Patch = PatchSpan;
    fn diff(&self, other: &Self) -> Option<Self::Patch> {
        Some(PatchSpan {
            common: self.common.diff(&other.common)?,
        })
    }
    fn apply(&mut self, patch: Self::Patch) -> ApplyResult {
        self.common.apply(patch.common)
    }
}

#[cfg(test)]
mod test {
    use super::{Common, Diff, PatchCommon, PatchSpan, Span};
    #[test]
    fn same() {
        let span1 = Span::default();
        let span2 = Span::default();
        assert_eq!(span1.diff(&span2), None)
    }

    #[test]
    fn different_id() {
        let mut span1 = Span::new(Common::new(None, Some("a".into()), Default::default()));
        let span2 = Span::new(Common::new(None, Some("b".into()), Default::default()));
        assert_ne!(span1, span2);
        let patch = span1.diff(&span2);
        assert_eq!(
            patch,
            Some(PatchSpan {
                common: PatchCommon {
                    id: Some(Some("b".into())),
                    children: Default::default()
                }
            })
        );
        span1.apply(patch.unwrap()).unwrap();
        assert_eq!(span1, span2);
    }
}
