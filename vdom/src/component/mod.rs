use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum ComponentNode {
    
}

pub trait Component {
    fn render() -> ComponentNode;
}
