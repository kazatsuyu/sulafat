use crate::{Attribute, ClosureId, Style, VariantIdent};
use serde_derive::{Deserialize, Serialize};
use sulafat_macros::VariantIdent;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, VariantIdent)]
#[serde(rename = "Attribute")]
pub enum RenderedAttribute {
    Id(String),
    OnClick(ClosureId),
    OnPointerMove(ClosureId),
    Style(Style),
}

impl<Msg> From<&Attribute<Msg>> for RenderedAttribute {
    fn from(attr: &Attribute<Msg>) -> Self {
        match attr {
            Attribute::Id(id) => RenderedAttribute::Id(id.clone()),
            Attribute::OnClick(handler) => RenderedAttribute::OnClick(handler.closure_id().clone()),
            Attribute::OnPointerMove(handler) => {
                RenderedAttribute::OnPointerMove(handler.closure_id().clone())
            }
            Attribute::Style(style) => RenderedAttribute::Style(style.clone()),
        }
    }
}

impl From<<Attribute<()> as VariantIdent>::Type> for RenderedAttributeVariantIdent {
    fn from(variant_ident: <Attribute<()> as VariantIdent>::Type) -> Self {
        match variant_ident {
            <Attribute<()> as VariantIdent>::Type::Id => Self::Id,
            <Attribute<()> as VariantIdent>::Type::OnClick => Self::OnClick,
            <Attribute<()> as VariantIdent>::Type::OnPointerMove => Self::OnPointerMove,
            <Attribute<()> as VariantIdent>::Type::Style => Self::Style,
        }
    }
}
