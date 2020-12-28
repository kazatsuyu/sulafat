pub type ApplyResult = Result<(), String>;

pub trait Diff {
    type Patch;
    fn diff(&self, other: &Self) -> Option<Self::Patch>;
    fn apply(&mut self, patch: Self::Patch) -> ApplyResult;
}