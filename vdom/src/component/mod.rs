use std::cell::UnsafeCell;

use super::Node;

#[derive(Debug)]
pub struct ComponentNode {
    rendered: UnsafeCell<Option<std::rc::Rc<Node>>>,
}

impl Clone for ComponentNode {
    fn clone(&self) -> Self {
        let rendered = unsafe { &*self.rendered.get() }.clone();
        Self {
            rendered: UnsafeCell::new(rendered),
        }
    }
}

impl PartialEq for ComponentNode {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}

impl Eq for ComponentNode {}

pub trait Component {
    fn render() -> ComponentNode;
}
