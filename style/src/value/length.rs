use std::fmt::{self, Display, Formatter};

use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Length {
    Em(f64),
    Px(f64),
    Vh(f64),
    Vw(f64),
}

impl Display for Length {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Length::Em(em) => write!(f, "{}em", em),
            Length::Px(px) => write!(f, "{}px", px),
            Length::Vh(vh) => write!(f, "{}vh", vh),
            Length::Vw(vw) => write!(f, "{}vw", vw),
        }
    }
}
