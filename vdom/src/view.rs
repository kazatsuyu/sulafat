use crate::{list::PatchListOp, ApplyResult, Diff, PatchNode};

use super::{ClosureId, Node};
use std::{
    any::Any,
    cell::UnsafeCell,
    fmt::{self, Debug, Formatter},
    rc::Rc,
};

pub struct CachedView<Msg> {
    key: Option<String>,
    view: Rc<dyn View<Msg>>,
    rendered: Option<Rc<UnsafeCell<Node<Msg>>>>,
}

impl<Msg> CachedView<Msg> {
    pub(crate) fn new<V: 'static + View<Msg>>(key: Option<String>, view: V) -> Self {
        Self {
            key,
            view: Rc::new(view),
            rendered: None.into(),
        }
    }

    pub fn key(&self) -> Option<&String> {
        self.key.as_ref()
    }

    pub(crate) unsafe fn rendered(&self) -> Option<&Node<Msg>> {
        Some(unsafe { &*self.rendered.as_ref()?.get() })
    }

    pub fn flat_len(&self) -> Option<usize> {
        unsafe { self.rendered() }?.flat_len()
    }

    pub(crate) fn render(&mut self) -> &mut Node<Msg> {
        let view = &self.view;
        let rendered = self
            .rendered
            .get_or_insert_with(|| Rc::new(UnsafeCell::new(view.render())));
        unsafe { &mut *rendered.get() }
    }

    pub(crate) fn is_full_rendered(&self) -> bool {
        unsafe { self.rendered() }
            .map(|rendered| rendered.is_full_rendered())
            .unwrap_or(false)
    }

    pub(crate) fn full_render(&mut self) -> &mut Node<Msg> {
        let node = self.render();
        node.full_render();
        node
    }

    pub(crate) fn is_different(&self, other: &Self) -> bool {
        self.key != other.key || self.view.as_any().type_id() != other.view.as_any().type_id()
    }

    pub(crate) fn share_cache_if_same(&self, other: &mut Self) -> bool {
        if self == other {
            other.rendered = self.rendered.clone();
            true
        } else {
            false
        }
    }

    pub(crate) fn add_patch(&mut self, patches: &mut Vec<PatchListOp<Msg>>) {
        self.render().add_patch(patches);
    }
}

impl<Msg> Clone for CachedView<Msg> {
    fn clone(&self) -> Self {
        let rendered = self.rendered.clone();
        Self {
            key: self.key.clone(),
            view: self.view.clone(),
            rendered,
        }
    }
}

impl<Msg> PartialEq for CachedView<Msg> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.view, &other.view) || &self.view == &other.view
    }
}

impl<Msg> Eq for CachedView<Msg> {}

impl<Msg> Debug for CachedView<Msg> {
    fn fmt(&self, _f: &mut Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

impl<Msg> Diff for CachedView<Msg> {
    type Patch = PatchNode<Msg>;
    fn diff(&self, other: &mut Self) -> Option<Self::Patch> {
        if self.share_cache_if_same(other) {
            None
        } else if self.is_different(other) {
            Some(PatchNode::Replace(other.full_render().clone()))
        } else {
            unsafe { self.rendered() }.unwrap().diff(other.render())
        }
    }
    fn apply(&mut self, _patch: Self::Patch) -> ApplyResult {
        unreachable!()
    }
}

pub trait View<Msg> {
    fn render(&self) -> Node<Msg>;
    fn is_same(&self, other: &dyn View<Msg>) -> bool;
    fn as_any(&self) -> &dyn Any;
}

impl<Msg> PartialEq for dyn View<Msg> {
    fn eq(&self, other: &Self) -> bool {
        self.is_same(other)
    }
}

impl<Msg> Eq for dyn View<Msg> {}

#[derive(Debug)]
pub struct Memo<Model, Msg, F> {
    f: F,
    id: ClosureId,
    model: Rc<Model>,
    _p: std::marker::PhantomData<Msg>,
}

impl<Model, Msg, F> Memo<Model, Msg, F>
where
    F: 'static + Fn(&Model) -> Node<Msg>,
    Model: 'static,
    Msg: 'static,
{
    pub fn new(f: F, model: Rc<Model>) -> Self {
        let id = ClosureId::new::<F, &Model, Node<Msg>>(&f);
        Self {
            f,
            id,
            model,
            _p: Default::default(),
        }
    }
}

impl<Model, Msg, F> View<Msg> for Memo<Model, Msg, F>
where
    F: 'static + Fn(&Model) -> Node<Msg>,
    Model: 'static + PartialEq,
    Msg: 'static,
{
    fn render(&self) -> Node<Msg> {
        (self.f)(&self.model)
    }
    fn is_same(&self, other: &dyn View<Msg>) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self == other
        } else {
            false
        }
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<Model, Msg, F> PartialEq for Memo<Model, Msg, F>
where
    F: Fn(&Model) -> Node<Msg>,
    Model: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.model == other.model
    }
}

impl<Model, Msg, F> Eq for Memo<Model, Msg, F>
where
    F: Fn(&Model) -> Node<Msg>,
    Model: PartialEq,
{
}
