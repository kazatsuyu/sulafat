use std::{any::Any, collections::HashMap, rc::Weak};

use crate::{ClosureId, Diff, Element, Node, PatchSingle};
use sulafat_macros::{Clone, PartialEq, Serialize};

use super::RenderedSingle;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Single<Msg> {
    Text(String),
    Element(Element<Msg>),
}

impl<Msg> Single<Msg> {
    pub fn key(&self) -> Option<&String> {
        if let Single::Element(element) = self {
            element.key().into()
        } else {
            None
        }
    }

    pub(crate) fn is_full_rendered(&self) -> bool {
        match self {
            Single::Text(_) => true,
            Single::Element(element) => element.is_full_rendered(),
        }
    }

    pub(crate) fn full_render(&mut self) {
        match self {
            Single::Text(_) => {}
            Single::Element(element) => element.full_render(),
        }
    }

    pub(crate) fn pick_handler(&self, handlers: &mut HashMap<ClosureId, Weak<dyn Any>>)
    where
        Msg: 'static,
    {
        match self {
            Single::Text(_) => {}
            Single::Element(element) => element.pick_handler(handlers),
        }
    }
}

impl<Msg> From<Single<Msg>> for Node<Msg> {
    fn from(node: Single<Msg>) -> Self {
        Node::Single(node)
    }
}

impl<Msg> Diff for Single<Msg> {
    type Patch = PatchSingle;
    fn diff(&self, other: &mut Self) -> Option<Self::Patch> {
        match (self, other) {
            (Single::Text(s), Single::Text(o)) => {
                if s == o {
                    None
                } else {
                    Some(PatchSingle::Replace(RenderedSingle::Text(o.clone())))
                }
            }
            (Single::Element(s), Single::Element(o)) => Some(s.diff(o)?.into()),
            (_, other) => Some(PatchSingle::Replace((&*other).into())),
        }
    }
}

impl<Msg> From<String> for Single<Msg> {
    fn from(s: String) -> Self {
        Single::Text(s)
    }
}

impl<Msg> From<&str> for Single<Msg> {
    fn from(s: &str) -> Self {
        Single::Text(s.into())
    }
}

impl<Msg> From<String> for Node<Msg> {
    fn from(s: String) -> Self {
        Single::from(s).into()
    }
}

impl<Msg> From<&str> for Node<Msg> {
    fn from(s: &str) -> Self {
        Single::from(s).into()
    }
}

impl<Msg> Eq for Single<Msg> {}

#[cfg(test)]
mod test {
    use crate::{single::RenderedSingle, Apply, Diff, Div, PatchSingle, Single};

    #[test]
    fn same_element() {
        let div1: Single<()> = Div::default().into();
        let mut div2 = Div::default().into();
        assert_eq!(div1.diff(&mut div2), None);
    }

    #[test]
    fn same_text() {
        let text1: Single<()> = "a".into();
        let mut text2 = "a".into();
        assert_eq!(text1.diff(&mut text2), None);
    }

    #[test]
    fn different() {
        let text: Single<()> = "a".into();
        let mut div = Div::default().into();
        assert_ne!(text, div);
        let patch = text.diff(&mut div);
        assert_eq!(patch, Some(PatchSingle::Replace((&div).into())));
        let mut rendered_text = RenderedSingle::from(&text);
        let rendered_div = RenderedSingle::from(&div);
        rendered_text.apply(patch.unwrap()).unwrap();
        assert_eq!(rendered_text, rendered_div);
    }
}
