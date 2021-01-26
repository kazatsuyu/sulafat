use crate::{Common, Diff, Element, Node, PatchSpan, Single};
use sulafat_macros::{Clone, PartialEq, Serialize};

#[derive(Default, Clone, Debug, PartialEq, Serialize)]
pub struct Span<Msg> {
    pub(crate) common: Common<Msg>,
}

impl<Msg> Span<Msg> {
    pub fn new(common: Common<Msg>) -> Self {
        Self { common }
    }

    pub fn common(&self) -> &Common<Msg> {
        &self.common
    }

    pub fn common_mut(&mut self) -> &mut Common<Msg> {
        &mut self.common
    }
}

impl<Msg> From<Span<Msg>> for Element<Msg> {
    fn from(span: Span<Msg>) -> Self {
        Element::Span(span)
    }
}

impl<Msg> From<Span<Msg>> for Single<Msg> {
    fn from(span: Span<Msg>) -> Self {
        Element::from(span).into()
    }
}

impl<Msg> From<Span<Msg>> for Node<Msg> {
    fn from(span: Span<Msg>) -> Self {
        Single::from(span).into()
    }
}

impl<Msg> Diff for Span<Msg> {
    type Patch = PatchSpan;
    fn diff(&self, other: &mut Self) -> Option<Self::Patch> {
        Some(PatchSpan {
            common: self.common.diff(&mut other.common)?,
        })
    }
}

impl<Msg> Eq for Span<Msg> {}

#[cfg(test)]
mod test {
    use crate::{
        element::rendered::RenderedSpan, id, Apply, Common, Diff, PatchAttributeListOp,
        PatchCommon, PatchSpan, RenderedAttribute, Span,
    };
    #[test]
    fn same() {
        let span1 = Span::<()>::default();
        let mut span2 = Span::default();
        assert_eq!(span1.diff(&mut span2), None)
    }

    #[test]
    fn different_id() {
        let span1 = Span::<()>::new(Common::new(
            None,
            vec![id("a".into())].into(),
            Default::default(),
        ));
        let mut span2 = Span::new(Common::new(
            None,
            vec![id("b".into())].into(),
            Default::default(),
        ));
        assert_ne!(span1, span2);
        let patch = span1.diff(&mut span2);
        assert_eq!(
            patch,
            Some(PatchSpan {
                common: PatchCommon {
                    attribute_list: vec![PatchAttributeListOp::Insert(RenderedAttribute::Id(
                        "b".into()
                    ))]
                    .into(),
                    children: Default::default()
                }
            })
        );
        let mut rendered_span1 = RenderedSpan::from(&span1);
        let rendered_span2 = RenderedSpan::from(&span2);
        rendered_span1.apply(patch.unwrap()).unwrap();
        assert_eq!(rendered_span1, rendered_span2);
    }
}
