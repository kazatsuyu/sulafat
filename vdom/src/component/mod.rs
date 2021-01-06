use std::cell::UnsafeCell;

use super::Node;

#[derive(Debug)]
pub struct ComponentNode<Msg> {
    rendered: UnsafeCell<Option<std::rc::Rc<Node<Msg>>>>,
}

impl<Msg> Clone for ComponentNode<Msg> {
    fn clone(&self) -> Self {
        let rendered = unsafe { &*self.rendered.get() }.clone();
        Self {
            rendered: UnsafeCell::new(rendered),
        }
    }
}

impl<Msg> PartialEq for ComponentNode<Msg> {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}

impl<Msg> Eq for ComponentNode<Msg> {}

pub trait Component<Msg> {
    fn render() -> ComponentNode<Msg>;
}
