use serde_derive::{Deserialize, Serialize};
use sulafat_style::{StyleRule, StyleSet};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Style {
    Static(String),
    Dynamic(Vec<StyleRule>),
}

pub trait ToStyle {
    fn to_style(&self) -> Style;
}

impl<S> ToStyle for S
where
    S: StyleSet,
{
    fn to_style(&self) -> Style {
        #[cfg(feature = "export-css")]
        {
            Style::Static(S::name())
        }
        #[cfg(not(feature = "export-css"))]
        {
            Style::Dynamic(Vec::from(S::rules()))
        }
    }
}

impl ToStyle for [StyleRule] {
    fn to_style(&self) -> Style {
        Style::Dynamic(self.to_owned())
    }
}
