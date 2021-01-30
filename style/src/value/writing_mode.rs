use std::fmt::{self, Display, Formatter};

use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum WritingMode {
    HorizontalTb,
    VerticalRl,
    VerticalLr,
    SidewayzRl,
    SidewayzLr,
}

impl WritingMode {
    pub fn is_horizontal(&self) -> bool {
        match self {
            WritingMode::HorizontalTb => true,
            WritingMode::VerticalRl => false,
            WritingMode::VerticalLr => false,
            WritingMode::SidewayzRl => false,
            WritingMode::SidewayzLr => false,
        }
    }

    pub fn is_vertical(&self) -> bool {
        match self {
            WritingMode::HorizontalTb => false,
            WritingMode::VerticalRl => true,
            WritingMode::VerticalLr => true,
            WritingMode::SidewayzRl => true,
            WritingMode::SidewayzLr => true,
        }
    }

    pub fn is_sideways(&self) -> bool {
        match self {
            WritingMode::HorizontalTb => false,
            WritingMode::VerticalRl => false,
            WritingMode::VerticalLr => false,
            WritingMode::SidewayzRl => true,
            WritingMode::SidewayzLr => true,
        }
    }
}

impl Display for WritingMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            WritingMode::HorizontalTb => f.write_str("horizontal-tb"),
            WritingMode::VerticalRl => f.write_str("vertical-rl"),
            WritingMode::VerticalLr => f.write_str("vertical-lr"),
            WritingMode::SidewayzRl => f.write_str("sideways-rl"),
            WritingMode::SidewayzLr => f.write_str("sideways-lr"),
        }
    }
}
