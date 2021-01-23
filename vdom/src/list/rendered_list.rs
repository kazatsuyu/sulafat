use std::ops::Deref;

use crate::{
    single::RenderedSingle, Apply, ApplyResult, List, Node, PatchList, PatchListOp, PatchSingle,
};
use serde_derive::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename = "List")]
pub struct RenderedList {
    list: Vec<RenderedSingle>,
}

impl RenderedList {
    fn append_list<Msg>(&mut self, list: &List<Msg>) {
        if let Some(flat_len) = list.flat_len() {
            if flat_len > self.len() {
                self.list.reserve(flat_len - self.len());
            }
        }
        for mut child in list.iter() {
            loop {
                match child {
                    Node::Single(single) => {
                        self.list.push(single.into());
                    }
                    Node::List(list) => {
                        self.append_list(list);
                    }
                    Node::CachedView(cached_view) => {
                        child = unsafe { cached_view.rendered() }.unwrap();
                        continue;
                    }
                }
                break;
            }
        }
    }
}

impl From<Vec<RenderedSingle>> for RenderedList {
    fn from(list: Vec<RenderedSingle>) -> Self {
        Self { list }
    }
}

impl Deref for RenderedList {
    type Target = Vec<RenderedSingle>;
    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl<Msg> From<&List<Msg>> for RenderedList {
    fn from(list: &List<Msg>) -> Self {
        let mut this = Self { list: vec![] };
        this.append_list(list);
        this
    }
}

impl Apply for RenderedList {
    type Patch = PatchList;
    fn apply(&mut self, patch: Self::Patch) -> ApplyResult {
        match patch {
            PatchList::All(patches) => {
                let mut prev = self.list.drain(..).map(|v| Some(v)).collect::<Vec<_>>();
                self.list.reserve(patches.len());
                for (index, patch) in patches.into_iter().enumerate() {
                    match patch {
                        PatchListOp::Nop => self.list.push(
                            prev[index]
                                .take()
                                .ok_or_else(|| String::from("元ノードの取得に失敗しました"))?
                                .into(),
                        ),
                        PatchListOp::From(index) => self.list.push(
                            prev[index]
                                .take()
                                .ok_or_else(|| String::from("元ノードの取得に失敗しました"))?
                                .into(),
                        ),
                        PatchListOp::Modify(patch) => {
                            let mut single = prev[index]
                                .take()
                                .ok_or_else(|| String::from("元ノードの取得に失敗しました"))?;
                            single.apply(patch)?;
                            self.list.push(single.into());
                        }
                        PatchListOp::FromModify(index, patch) => {
                            let mut single = prev[index]
                                .take()
                                .ok_or_else(|| String::from("元ノードの取得に失敗しました"))?;
                            single.apply(patch)?;
                            self.list.push(single.into());
                        }
                        PatchListOp::New(single) => self.list.push(single.into()),
                    }
                }
            }
            PatchList::Entries(len, entries) => {
                if len > self.len() {
                    self.list.reserve(len - self.len());
                }
                self.list.truncate(len);
                for (index, patch) in entries {
                    if index >= self.len() {
                        if let PatchSingle::Replace(single) = patch {
                            self.list.push(single.into())
                        } else {
                            return Err("不正なパッチです".into());
                        }
                    } else {
                        self.list[index].apply(patch)?;
                    }
                }
            }
            PatchList::Truncate(len) => {
                self.list.truncate(len);
            }
        }
        Ok(())
    }
}
