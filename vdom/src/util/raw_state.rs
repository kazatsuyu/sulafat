use num::{NumCast, PrimInt};
use std::ops::BitXorAssign;

pub(crate) struct RawStateOf<const SIZE: usize>;

pub(crate) trait RawState {
    type Type: Default + NumCast + PrimInt + BitXorAssign;
}
impl RawState for RawStateOf<1> {
    type Type = u8;
}
impl RawState for RawStateOf<2> {
    type Type = u16;
}
impl RawState for RawStateOf<4> {
    type Type = u32;
}
impl RawState for RawStateOf<8> {
    type Type = u64;
}
impl RawState for RawStateOf<16> {
    type Type = u128;
}
