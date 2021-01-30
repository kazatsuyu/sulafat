use std::fmt::{self, Formatter};

use fmt::Display;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Parcentage(pub f64);

impl Display for Parcentage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.0)
    }
}
