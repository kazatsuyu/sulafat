pub mod raw_state;
pub mod steal_hasher;

pub(crate) use raw_state::{RawState, RawStateOf};
pub(crate) use steal_hasher::StealHasher;

use std::{any::TypeId, mem::forget};

unsafe fn unsafe_cast<T, U>(t: &T) -> &U {
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

unsafe fn unsafe_force_cast<T, U>(t: T) -> U {
    let ptr = &t as *const T as *const U;
    let u = unsafe { ptr.read() };
    forget(t);
    u
}

pub(crate) fn force_cast<T, U>(t: T) -> Result<U, T>
where
    T: 'static,
    U: 'static,
{
    if TypeId::of::<T>() == TypeId::of::<U>() {
        Ok(unsafe { unsafe_force_cast(t) })
    } else {
        Err(t)
    }
}

#[cfg(target_arch = "wasm32")]
#[allow(dead_code)]
pub(crate) fn debug(value: &wasm_bindgen::JsValue) {
    // NOTE: If not enclosed in an unsafe block, the rust-analyzer gives an unsafe error,
    // but if enclosed in an unsafe block, the warning unused_unsafe appears.
    // To avoid this, enclose it in an unsafe block and add #[allow(unused_unsafe)].
    #[allow(unused_unsafe)]
    unsafe {
        web_sys::console::debug_1(value)
    }
}
