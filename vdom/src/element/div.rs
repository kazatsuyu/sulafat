use crate::{Common, Diff, Element, Node, PatchDiv, Single};
use sulafat_macros::Serialize;

#[derive(Default, Debug, Serialize)]
pub struct Div<Msg> {
    pub(crate) common: Common<Msg>,
}

impl<Msg> Div<Msg> {
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

impl<Msg> From<Div<Msg>> for Element<Msg> {
    fn from(div: Div<Msg>) -> Self {
        Element::Div(div)
    }
}

impl<Msg> From<Div<Msg>> for Single<Msg> {
    fn from(div: Div<Msg>) -> Self {
        Element::from(div).into()
    }
}

impl<Msg> From<Div<Msg>> for Node<Msg> {
    fn from(div: Div<Msg>) -> Self {
        Single::from(div).into()
    }
}

impl<Msg> Diff for Div<Msg> {
    type Patch = PatchDiv;
    fn diff(&self, other: &mut Self) -> Option<Self::Patch> {
        Some(PatchDiv {
            common: self.common.diff(&mut other.common)?,
        })
    }
}

impl<Msg> Clone for Div<Msg> {
    fn clone(&self) -> Self {
        Self {
            common: self.common.clone(),
        }
    }
}

impl<Msg> PartialEq for Div<Msg> {
    fn eq(&self, other: &Self) -> bool {
        self.common == other.common
    }
}

impl<Msg> Eq for Div<Msg> {}

#[cfg(test)]
mod test {
    use crate::{
        element::rendered::{RenderedAttribute, RenderedDiv},
        id, Apply, Common, Diff, Div, PatchAttributeListOp, PatchCommon, PatchDiv,
    };
    #[test]
    fn same() {
        let div1 = Div::<()>::default();
        let mut div2 = Div::default();
        assert_eq!(div1.diff(&mut div2), None)
    }

    #[test]
    fn different_id() {
        let div1 = Div::<()>::new(Common::new(
            None,
            vec![id("a".into())].into(),
            Default::default(),
        ));
        let mut div2 = Div::new(Common::new(
            None,
            vec![id("b".into())].into(),
            Default::default(),
        ));
        assert_ne!(div1, div2);
        let patch = div1.diff(&mut div2);
        assert_eq!(
            patch,
            Some(PatchDiv {
                common: PatchCommon {
                    attribute_list: vec![PatchAttributeListOp::Insert(RenderedAttribute::Id(
                        "b".into()
                    ))]
                    .into(),
                    children: Default::default()
                }
            })
        );
        let mut rendered_div1 = RenderedDiv::from(&div1);
        let rendered_div2 = RenderedDiv::from(&div2);
        rendered_div1.apply(patch.unwrap()).unwrap();
        assert_eq!(rendered_div1, rendered_div2);
    }
}
