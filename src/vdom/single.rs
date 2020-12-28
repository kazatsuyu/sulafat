use super::{ApplyResult, Diff, Element, Node, PatchElement, PatchNode};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Single {
    Text(String),
    Element(Element),
}

impl Single {
    pub fn key(&self) -> Option<&String> {
        if let Single::Element(element) = self {
            element.key().into()
        } else {
            None
        }
    }
}

impl From<Single> for Node {
    fn from(node: Single) -> Self {
        Node::Single(node)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PatchSingle {
    Replace(Single),
    Element(PatchElement),
}

impl From<PatchSingle> for PatchNode {
    fn from(patch: PatchSingle) -> Self {
        match patch {
            PatchSingle::Replace(single) => PatchNode::Replace(single.into()),
            _ => PatchNode::Single(patch),
        }
    }
}

impl Diff for Single {
    type Patch = PatchSingle;
    fn diff(&self, other: &Self) -> Option<Self::Patch> {
        match (self, other) {
            (Single::Text(s), Single::Text(o)) => {
                if s == o {
                    None
                } else {
                    Some(PatchSingle::Replace(Single::Text(o.clone())))
                }
            }
            (Single::Element(s), Single::Element(o)) => Some(s.diff(o)?.into()),
            _ => Some(PatchSingle::Replace(other.clone())),
        }
    }
    fn apply(&mut self, patch: Self::Patch) -> ApplyResult {
        match patch {
            PatchSingle::Replace(node) => *self = node,
            PatchSingle::Element(patch) => {
                if let Single::Element(element) = self {
                    element.apply(patch)?
                } else {
                    return Err("Elementではありません".into());
                }
            }
        }
        Ok(())
    }
}

impl From<String> for Single {
    fn from(s: String) -> Self {
        Single::Text(s)
    }
}

impl From<&str> for Single {
    fn from(s: &str) -> Self {
        Single::Text(s.into())
    }
}

impl From<String> for Node {
    fn from(s: String) -> Self {
        Single::from(s).into()
    }
}

impl From<&str> for Node {
    fn from(s: &str) -> Self {
        Single::from(s).into()
    }
}

#[cfg(test)]
mod test {
    use super::{super::Div, Diff, PatchSingle, Single};

    #[test]
    fn same_element() {
        let div1: Single = Div::default().into();
        let div2 = Div::default().into();
        assert_eq!(div1.diff(&div2), None);
    }

    #[test]
    fn same_text() {
        let text1: Single = "a".into();
        let text2 = "a".into();
        assert_eq!(text1.diff(&text2), None);
    }

    #[test]
    fn different() {
        let mut text: Single = "a".into();
        let div = Div::default().into();
        assert_ne!(text, div);
        let patch = text.diff(&div);
        assert_eq!(patch, Some(PatchSingle::Replace(div.clone())));
        text.apply(patch.unwrap()).unwrap();
        assert_eq!(text, div);
    }
}
