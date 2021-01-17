use crate::{
    Attribute, ClosureId, Common, Diff, Div, List, Node, PatchElement, Single, Span, VariantIdent,
};
use sulafat_macros::Serialize;

use std::{any::Any, collections::HashMap, rc::Weak};
use sulafat_macros::VariantIdent;

#[derive(Debug, Eq, Serialize, VariantIdent)]
pub enum Element<Msg> {
    Div(Div<Msg>),
    Span(Span<Msg>),
}

impl<Msg> Element<Msg> {
    fn common(&self) -> &Common<Msg> {
        match self {
            Element::Div(div) => div.common(),
            Element::Span(span) => span.common(),
        }
    }
    fn common_mut(&mut self) -> &mut Common<Msg> {
        match self {
            Element::Div(div) => div.common_mut(),
            Element::Span(span) => span.common_mut(),
        }
    }

    pub fn id(&self) -> Option<&String> {
        let index = self
            .common()
            .attribute_list
            .binary_search_by(|v| {
                v.variant_ident()
                    .cmp(&<Attribute<Msg> as VariantIdent>::Type::Id)
            })
            .ok()?;
        if let Attribute::Id(id) = &self.common().attribute_list[index] {
            Some(id)
        } else {
            unreachable!()
        }
    }

    pub fn children(&self) -> &List<Msg> {
        &self.common().children
    }

    pub fn children_mut(&mut self) -> &mut List<Msg> {
        &mut self.common_mut().children
    }

    pub fn key(&self) -> &Option<String> {
        &self.common().key
    }

    pub(crate) fn is_full_rendered(&self) -> bool {
        self.common().children.is_full_rendered()
    }

    pub(crate) fn full_render(&mut self) {
        self.common_mut().children.full_render()
    }

    pub(crate) fn pick_handler(&self, handlers: &mut HashMap<ClosureId, Weak<dyn Any>>)
    where
        Msg: 'static,
    {
        for attr in self.common().attribute_list.iter() {
            attr.pick_handler(handlers)
        }
        self.common().children.pick_handler(handlers)
    }
}

impl<Msg> From<Element<Msg>> for Single<Msg> {
    fn from(element: Element<Msg>) -> Self {
        Single::Element(element)
    }
}

impl<Msg> From<Element<Msg>> for Node<Msg> {
    fn from(element: Element<Msg>) -> Self {
        Single::from(element).into()
    }
}

impl<Msg> Diff for Element<Msg> {
    type Patch = PatchElement;
    fn diff(&self, other: &mut Self) -> Option<Self::Patch> {
        if self.key() != other.key() {
            return Some(PatchElement::Replace((&*other).into()));
        }
        Some(match (self, other) {
            (Element::Div(div1), Element::Div(div2)) => div1.diff(div2)?.into(),
            (Element::Span(div1), Element::Span(div2)) => div1.diff(div2)?.into(),
            (_, other) => PatchElement::Replace((&*other).into()),
        })
    }
}

impl<Msg> Clone for Element<Msg> {
    fn clone(&self) -> Self {
        match self {
            Element::Div(div) => Element::Div(div.clone()),
            Element::Span(span) => Element::Span(span.clone()),
        }
    }
}

impl<Msg> PartialEq for Element<Msg> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Element::Div(this), Element::Div(other)) => this == other,
            (Element::Span(this), Element::Span(other)) => this == other,
            _ => false,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        element::rendered::RenderedAttribute, id, Apply, Common, Diff, Div, Element,
        PatchAttributeListOp, PatchCommon, PatchDiv, PatchElement, PatchSpan, RenderedElement,
        Span,
    };

    #[test]
    fn div_same() {
        let div1: Element<()> = Div::default().into();
        let mut div2 = Div::default().into();
        assert_eq!(div1.diff(&mut div2), None);
    }

    #[test]
    fn span_same() {
        let span1: Element<()> = Span::default().into();
        let mut span2 = Span::default().into();
        assert_eq!(span1.diff(&mut span2), None);
    }

    #[test]
    fn div_different_id() {
        let div1: Element<()> = Div::new(Common::new(
            None,
            vec![id("a".into())].into(),
            Default::default(),
        ))
        .into();
        let mut div2 = Div::new(Common::new(
            None,
            vec![id("b".into())].into(),
            Default::default(),
        ))
        .into();
        assert_ne!(div1, div2);
        let patch = div1.diff(&mut div2);
        assert_eq!(
            patch,
            Some(
                PatchDiv {
                    common: PatchCommon {
                        attribute_list: vec![PatchAttributeListOp::Insert(RenderedAttribute::Id(
                            "b".into()
                        ))]
                        .into(),
                        children: Default::default()
                    }
                }
                .into()
            )
        );
        let mut rendered_div1 = RenderedElement::from(&div1);
        let rendered_div2 = RenderedElement::from(&div2);
        rendered_div1.apply(patch.unwrap()).unwrap();
        assert_eq!(rendered_div1, rendered_div2);
    }

    #[test]
    fn span_different_id() {
        let span1: Element<()> = Span::new(Common::new(
            None,
            vec![id("a".into())].into(),
            Default::default(),
        ))
        .into();
        let mut span2 = Span::new(Common::new(
            None,
            vec![id("b".into())].into(),
            Default::default(),
        ))
        .into();
        assert_ne!(span1, span2);
        let patch = span1.diff(&mut span2);
        assert_eq!(
            patch,
            Some(
                PatchSpan {
                    common: PatchCommon {
                        attribute_list: vec![PatchAttributeListOp::Insert(RenderedAttribute::Id(
                            "b".into()
                        ))]
                        .into(),
                        children: Default::default()
                    }
                }
                .into()
            )
        );
        let mut rendered_span1 = RenderedElement::from(&span1);
        let rendered_span2 = RenderedElement::from(&span2);
        rendered_span1.apply(patch.unwrap()).unwrap();
        assert_eq!(rendered_span1, rendered_span2);
    }

    #[test]
    fn div_different_key() {
        let div1: Element<()> = Div::new(Common::new(
            Some("a".into()),
            vec![].into(),
            Default::default(),
        ))
        .into();
        let mut div2 = Div::new(Common::new(
            Some("b".into()),
            vec![].into(),
            Default::default(),
        ))
        .into();
        assert_ne!(div1, div2);
        let patch = div1.diff(&mut div2);
        assert_eq!(patch, Some(PatchElement::Replace((&div2).into())));
        let mut rendered_div1 = RenderedElement::from(&div1);
        let rendered_div2 = RenderedElement::from(&div2);
        rendered_div1.apply(patch.unwrap()).unwrap();
        assert_eq!(rendered_div1, rendered_div2);
    }

    #[test]
    fn span_different_key() {
        let span1: Element<()> = Span::new(Common::new(
            Some("a".into()),
            vec![].into(),
            Default::default(),
        ))
        .into();
        let mut span2 = Span::new(Common::new(
            Some("b".into()),
            vec![].into(),
            Default::default(),
        ))
        .into();
        assert_ne!(span1, span2);
        let patch = span1.diff(&mut span2);
        assert_eq!(patch, Some(PatchElement::Replace((&span2).into())));
        let mut rendered_span1 = RenderedElement::from(&span1);
        let rendered_span2 = RenderedElement::from(&span2);
        rendered_span1.apply(patch.unwrap()).unwrap();
        assert_eq!(rendered_span1, rendered_span2);
    }

    #[test]
    fn element_different_tag() {
        let div: Element<()> = Div::default().into();
        let mut span = Span::default().into();
        assert_ne!(div, span);
        let patch = div.diff(&mut span);
        assert_eq!(patch, Some(PatchElement::Replace((&span).into())));
        let mut rendered_div = RenderedElement::from(&div);
        let rendered_span = RenderedElement::from(&span);
        rendered_div.apply(patch.unwrap()).unwrap();
        assert_eq!(rendered_div, rendered_span);
    }
}
