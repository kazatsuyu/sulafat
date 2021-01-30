use std::fmt::{self, Display};

use fmt::Formatter;
use serde_derive::{Deserialize, Serialize};

use crate::{Length, Parcentage};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LengthOrPercentage {
    Length(Length),
    Parcentage(Parcentage),
}

impl Display for LengthOrPercentage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LengthOrPercentage::Length(length) => Display::fmt(length, f),
            LengthOrPercentage::Parcentage(parcentage) => Display::fmt(parcentage, f),
        }
    }
}
