use crate::util::{safe_cast, RawState, RawStateOf, StealHasher};
use serde_derive::{Deserialize, Serialize};
use std::hash::Hash;

const SIZE_OF_TYPEID: usize = std::mem::size_of::<std::any::TypeId>();

type TypeIdRawType = <RawStateOf<SIZE_OF_TYPEID> as RawState>::Type;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub struct TypeId {
    t: TypeIdRawType,
}

impl TypeId {
    pub(crate) fn of<T: 'static>() -> Self {
        let type_id = std::any::TypeId::of::<T>();
        let mut hasher = StealHasher::<SIZE_OF_TYPEID>::default();
        type_id.hash(&mut hasher);
        Self { t: hasher.get() }
    }
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ClosureId {
    TypeId(TypeId),
    FnPtr(usize),
}

impl ClosureId {
    pub(crate) fn new<F: 'static + Fn(Args) -> Output, Args, Output>(f: &F) -> Self
    where
        Args: 'static,
        Output: 'static,
    {
        let id = TypeId::of::<F>();
        if let Some(&fn_ptr) = safe_cast::<F, fn(Args) -> Output>(&f) {
            ClosureId::FnPtr(fn_ptr as usize)
        } else {
            ClosureId::TypeId(id)
        }
    }
}
