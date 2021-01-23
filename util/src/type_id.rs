use crate::{RawState, RawStateOf, StealHasher};
use serde_derive::{Deserialize, Serialize};
use std::hash::Hash;

const SIZE_OF_TYPEID: usize = std::mem::size_of::<std::any::TypeId>();

type TypeIdRawType = <RawStateOf<SIZE_OF_TYPEID> as RawState>::Type;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
pub struct TypeId {
    t: TypeIdRawType,
}

impl TypeId {
    pub fn of<T: 'static + ?Sized>() -> Self {
        let type_id = std::any::TypeId::of::<T>();
        let mut hasher = StealHasher::<SIZE_OF_TYPEID>::default();
        type_id.hash(&mut hasher);
        Self { t: hasher.get() }
    }

    pub fn inner(&self) -> TypeIdRawType {
        self.t
    }
}
