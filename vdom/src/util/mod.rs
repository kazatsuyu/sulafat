pub mod raw_state;
pub mod steal_hasher;

pub(crate) use raw_state::{RawState, RawStateOf};
pub(crate) use steal_hasher::StealHasher;

use std::any::TypeId;

pub(crate) unsafe fn unsafe_cast<T, U>(t: &T) -> &U {
    let ptr = t as *const T as *const U;
    unsafe { &*ptr }
}

pub(crate) fn safe_cast<T, U>(t: &T) -> Option<&U>
where
    T: 'static,
    U: 'static,
{
    if TypeId::of::<T>() == TypeId::of::<U>() {
        Some(unsafe { unsafe_cast(t) })
    } else {
        None
    }
}

pub(crate) unsafe fn reinterpret_cast<T, U>(t: T) -> U {
    let ptr = &t as *const T as *const U;
    let u = unsafe { ptr.read() };
    std::mem::forget(t);
    u
}
