use std::{hash::Hasher, mem::size_of};

use num::{NumCast, PrimInt};

use crate::{RawState, RawStateOf};

#[derive(Default)]
pub(crate) struct StealHasher<const SIZE: usize>(<RawStateOf<SIZE> as RawState>::Type, u32)
where
    RawStateOf<SIZE>: RawState;

impl<const SIZE: usize> StealHasher<SIZE>
where
    RawStateOf<SIZE>: RawState,
{
    pub(crate) fn get(&self) -> <RawStateOf<SIZE> as RawState>::Type {
        self.0
    }
}

impl<const SIZE: usize> Hasher for StealHasher<SIZE>
where
    RawStateOf<SIZE>: RawState,
{
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 ^= <<RawStateOf<SIZE> as RawState>::Type as NumCast>::from(b)
                .unwrap()
                .rotate_left(self.1);
            self.1 = (self.1 + 8) % (SIZE as u32 * 8);
        }
    }
    fn write_u16(&mut self, i: u16) {
        if SIZE <= size_of::<u16>() {
            self.0 ^= <<RawStateOf<SIZE> as RawState>::Type as NumCast>::from(i)
                .unwrap()
                .rotate_left(self.1);
            self.1 = (self.1 + 8 * size_of::<u16>() as u32) % (SIZE as u32 * 8);
        } else {
            self.write(&i.to_le_bytes())
        }
    }
    fn write_u32(&mut self, i: u32) {
        if SIZE <= size_of::<u32>() {
            self.0 ^= <<RawStateOf<SIZE> as RawState>::Type as NumCast>::from(i)
                .unwrap()
                .rotate_left(self.1);
            self.1 = (self.1 + 8 * size_of::<u32>() as u32) % (SIZE as u32 * 8);
        } else {
            self.write(&i.to_le_bytes())
        }
    }
    fn write_u64(&mut self, i: u64) {
        if SIZE <= size_of::<u64>() {
            self.0 ^= <<RawStateOf<SIZE> as RawState>::Type as NumCast>::from(i)
                .unwrap()
                .rotate_left(self.1);
            self.1 = (self.1 + 8 * size_of::<u64>() as u32) % (SIZE as u32 * 8);
        } else {
            self.write(&i.to_le_bytes())
        }
    }
    fn write_u128(&mut self, i: u128) {
        if SIZE <= size_of::<u128>() {
            self.0 ^= <<RawStateOf<SIZE> as RawState>::Type as NumCast>::from(i)
                .unwrap()
                .rotate_left(self.1);
            self.1 = (self.1 + 8 * size_of::<u128>() as u32) % (SIZE as u32 * 8);
        } else {
            self.write(&i.to_le_bytes())
        }
    }
    fn finish(&self) -> u64 {
        unimplemented!()
    }
}
