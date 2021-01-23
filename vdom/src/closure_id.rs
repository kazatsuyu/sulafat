use crate::util::safe_cast;
use serde_derive::{Deserialize, Serialize};
use std::hash::Hash;
use sulafat_util::TypeId;

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
    pub(crate) fn with_data<F: 'static + Fn(Data, Args) -> Output, Data, Args, Output>(
        f: &F,
    ) -> Self
    where
        Data: 'static,
        Args: 'static,
        Output: 'static,
    {
        let id = TypeId::of::<F>();
        if let Some(&fn_ptr) = safe_cast::<F, fn(Data, Args) -> Output>(&f) {
            ClosureId::FnPtr(fn_ptr as usize)
        } else {
            ClosureId::TypeId(id)
        }
    }
}
