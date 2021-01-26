use crate::{Attribute, ClosureId, Style, VariantIdent};
use serde_derive::{Deserialize, Serialize};
use sulafat_macros::VariantIdent;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, VariantIdent)]
#[serde(rename = "Attribute")]
#[from(<Attribute<()> as VariantIdent>::Type)]
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
