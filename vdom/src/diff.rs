pub trait Diff {
    type Patch;
    fn diff(&self, other: &mut Self) -> Option<Self::Patch>;
}

pub type ApplyResult = Result<(), String>;

pub trait Apply {
    type Patch;
    fn apply(&mut self, patch: Self::Patch) -> ApplyResult;
}
