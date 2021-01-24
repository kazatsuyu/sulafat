use crate::{AttributeList, Diff, List, PatchAttributeList, PatchCommon};
use sulafat_macros::Serialize;

#[derive(Default, Debug, Serialize)]
pub struct Common<Msg> {
    #[serde(skip)]
    pub(crate) key: Option<String>,
    pub(crate) attribute_list: AttributeList<Msg>,
    pub(crate) children: List<Msg>,
}

impl<Msg> Common<Msg> {
    pub fn new(
        key: Option<String>,
        attribute_list: AttributeList<Msg>,
        children: List<Msg>,
    ) -> Self {
        Self {
            key,
            attribute_list,
            children,
        }
    }
}

impl<Msg> Diff for Common<Msg> {
    type Patch = PatchCommon;
    fn diff(&self, other: &mut Self) -> Option<Self::Patch> {
        match (
            self.attribute_list.diff(&mut other.attribute_list),
            self.children.diff(&mut other.children),
        ) {
            (None, None) => None,
            (attribute_list, children) => Some(PatchCommon {
                attribute_list: attribute_list.unwrap_or_else(|| PatchAttributeList::default()),
                children,
            }),
        }
    }
}

impl<Msg> Clone for Common<Msg> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            attribute_list: self.attribute_list.clone(),
            children: self.children.clone(),
        }
    }
}

impl<Msg> PartialEq for Common<Msg> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.attribute_list == other.attribute_list
            && self.children == other.children
    }
}

impl<Msg> Eq for Common<Msg> {}

#[cfg(test)]
mod test {
    use crate::{
        element::rendered::RenderedCommon, id, Apply, Common, Diff, PatchAttributeListOp,
        PatchCommon, RenderedAttribute,
    };
    #[test]
    fn same() {
        let common1 = Common::<()>::default();
        let mut common2 = Common::default();
        assert_eq!(common1.diff(&mut common2), None)
    }

    #[test]
    fn different_id() {
        let common1 = Common::<()>::new(None, vec![id("a".into())].into(), Default::default());
        let mut common2 = Common::new(None, vec![id("b".into())].into(), Default::default());
        let patch = common1.diff(&mut common2);
        assert_eq!(
            patch,
            Some(PatchCommon {
                attribute_list: vec![PatchAttributeListOp::Insert(RenderedAttribute::Id(
                    "b".into()
                ))]
                .into(),
                children: Default::default()
            })
        );
        let mut rendered_common1 = RenderedCommon::from(&common1);
        let rendered_common2 = RenderedCommon::from(&common2);
        rendered_common1.apply(patch.unwrap()).unwrap();
        assert_eq!(rendered_common1, rendered_common2);
    }
}
