mod div;
mod span;

use super::{ApplyResult, Diff, List, Node, PatchList, PatchNode, PatchSingle, Single};
pub use div::{Div, PatchDiv};
pub use span::{PatchSpan, Span};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Element {
    Div(Div),
    Span(Span),
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum Tag {
    Div,
    Span,
}

impl Element {
    fn common(&self) -> &Common {
        match self {
            Element::Div(div) => div.common(),
            Element::Span(span) => span.common(),
        }
    }
    fn common_mut(&mut self) -> &mut Common {
        match self {
            Element::Div(div) => div.common_mut(),
            Element::Span(span) => span.common_mut(),
        }
    }
    pub fn tag(&self) -> Tag {
        match self {
            Element::Div(_) => Tag::Div,
            Element::Span(_) => Tag::Span,
        }
    }

    pub fn id(&self) -> &Option<String> {
        &self.common().id
    }

    pub fn children(&self) -> &List {
        &self.common().children
    }

    pub fn children_mut(&mut self) -> &mut List {
        &mut self.common_mut().children
    }

    pub fn key(&self) -> &Option<String> {
        &self.common().key
    }
}

impl From<Element> for Single {
    fn from(element: Element) -> Self {
        Single::Element(element)
    }
}

impl From<Element> for Node {
    fn from(element: Element) -> Self {
        Single::from(element).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PatchElement {
    Replace(Element),
    Div(PatchDiv),
    Span(PatchSpan),
}

impl From<PatchElement> for PatchSingle {
    fn from(patch: PatchElement) -> Self {
        match patch {
            PatchElement::Replace(single) => PatchSingle::Replace(single.into()),
            _ => PatchSingle::Element(patch),
        }
    }
}

impl From<PatchElement> for PatchNode {
    fn from(patch: PatchElement) -> Self {
        PatchSingle::from(patch).into()
    }
}

impl Diff for Element {
    type Patch = PatchElement;
    fn diff(&self, other: &Self) -> Option<Self::Patch> {
        use Element::*;
        if self.key() != other.key() {
            return Some(PatchElement::Replace(other.clone()));
        }
        Some(match (self, other) {
            (Div(div1), Div(div2)) => div1.diff(div2)?.into(),
            (Span(div1), Span(div2)) => div1.diff(div2)?.into(),
            _ => PatchElement::Replace(other.clone()),
        })
    }

    fn apply(&mut self, patch: Self::Patch) -> ApplyResult {
        use PatchElement::*;
        macro_rules! apply_element {
            ($($id:ident),*) => {
                match patch {
                    Replace(element) => {
                        *self = element;
                        return Ok(());
                    }
                    $($id(patch) => {
                        if let Element::$id(element) = self {
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

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Common {
    pub(in crate::vdom) key: Option<String>,
    pub(in crate::vdom) id: Option<String>,
    pub(in crate::vdom) children: List,
}

impl Common {
    pub fn new(key: Option<String>, id: Option<String>, children: List) -> Self {
        Self { key, id, children }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PatchCommon {
    pub(in crate::vdom) id: Option<Option<String>>,
    pub(in crate::vdom) children: Option<PatchList>,
}

impl Diff for Common {
    type Patch = PatchCommon;
    fn diff(&self, other: &Self) -> Option<Self::Patch> {
        if self != other {
            Some(PatchCommon {
                id: if self.id != other.id {
                    Some(other.id.clone())
                } else {
                    None
                },
                children: self.children.diff(&other.children),
            })
        } else {
            None
        }
    }
    fn apply(&mut self, patch: Self::Patch) -> ApplyResult {
        if let Some(id) = patch.id {
            self.id = id;
        }
        if let Some(patch) = patch.children {
            self.children.apply(patch)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::{Common, Diff, Div, Element, PatchCommon, PatchDiv, PatchSpan, Span};
    #[test]
    fn common_same() {
        let common1 = Common::default();
        let common2 = Common::default();
        assert_eq!(common1.diff(&common2), None)
    }

    #[test]
    fn common_different_id() {
        let mut common1 = Common::new(None, Some("a".into()), Default::default());
        let common2 = Common::new(None, Some("b".into()), Default::default());
        let patch = common1.diff(&common2);
        assert_eq!(
            patch,
            Some(PatchCommon {
                id: Some(Some("b".into())),
                children: Default::default()
            })
        );
        common1.apply(patch.unwrap()).unwrap();
        assert_eq!(common1, common2);
    }

    #[test]
    fn div_same() {
        let div1: Element = Div::default().into();
        let div2 = Div::default().into();
        assert_eq!(div1.diff(&div2), None);
    }

    #[test]
    fn span_same() {
        let span1: Element = Span::default().into();
        let span2 = Span::default().into();
        assert_eq!(span1.diff(&span2), None);
    }

    #[test]
    fn div_different_id() {
        let mut div1 = Element::Div(Div::new(Common::new(
            None,
            Some("a".into()),
            Default::default(),
        )));
        let div2 = Element::Div(Div::new(Common::new(
            None,
            Some("b".into()),
            Default::default(),
        )));
        assert_ne!(div1, div2);
        let patch = div1.diff(&div2);
        assert_eq!(
            patch,
            Some(
                PatchDiv {
                    common: PatchCommon {
                        id: Some(Some("b".into())),
                        children: Default::default()
                    }
                }
                .into()
            )
        );
        div1.apply(patch.unwrap()).unwrap();
        assert_eq!(div1, div2);
    }

    #[test]
    fn span_different_id() {
        let mut span1: Element =
            Span::new(Common::new(None, Some("a".into()), Default::default())).into();
        let span2 = Span::new(Common::new(None, Some("b".into()), Default::default())).into();
        assert_ne!(span1, span2);
        let patch = span1.diff(&span2);
        assert_eq!(
            patch,
            Some(
                PatchSpan {
                    common: PatchCommon {
                        id: Some(Some("b".into())),
                        children: Default::default()
                    }
                }
                .into()
            )
        );
        span1.apply(patch.unwrap()).unwrap();
        assert_eq!(span1, span2);
    }

    #[test]
    fn div_different_key() {
        let mut div1: Element =
            Div::new(Common::new(Some("a".into()), None, Default::default())).into();
        let div2 = Div::new(Common::new(Some("b".into()), None, Default::default())).into();
        assert_ne!(div1, div2);
        let patch = div1.diff(&div2);
        assert_eq!(patch, Some(super::PatchElement::Replace(div2.clone())));
        div1.apply(patch.unwrap()).unwrap();
        assert_eq!(div1, div2);
    }

    #[test]
    fn span_different_key() {
        let mut span1: Element =
            Span::new(Common::new(Some("a".into()), None, Default::default())).into();
        let span2 = Span::new(Common::new(Some("b".into()), None, Default::default())).into();
        assert_ne!(span1, span2);
        let patch = span1.diff(&span2);
        assert_eq!(patch, Some(super::PatchElement::Replace(span2.clone())));
        span1.apply(patch.unwrap()).unwrap();
        assert_eq!(span1, span2);
    }

    #[test]
    fn element_different_tag() {
        let mut div: Element = Div::default().into();
        let span = Span::default().into();
        assert_ne!(div, span);
        let patch = div.diff(&span);
        assert_eq!(patch, Some(super::PatchElement::Replace(span.clone())));
        div.apply(patch.unwrap()).unwrap();
        assert_eq!(div, span);
    }
}
