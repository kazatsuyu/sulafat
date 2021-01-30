mod value;

use serde_derive::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    fmt::{self, Display, Formatter, Write},
};
pub use value::{Length, LengthOrPercentage, Parcentage, WritingMode};

// やりたいこと
// * CSSを自動で出力（Web)
// * テーマによる動的なスタイル切り替え
//
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StyleRule {
    Left(LengthOrPercentage),
    Right(LengthOrPercentage),
    WritingMode(WritingMode),
}

impl Display for StyleRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            StyleRule::Left(value) => {
                write!(f, "left:{};", value)
            }
            StyleRule::Right(value) => {
                write!(f, "right:{};", value)
            }
            StyleRule::WritingMode(value) => write!(f, "writing-mode:{};", value),
        }
    }
}

pub trait StyleSet: 'static {
    fn rules() -> &'static [StyleRule];
    fn name() -> String;
    fn render<R: StyleRenderer>(renderer: &mut R) {
        for rule in Self::rules() {
            renderer.render(rule);
        }
    }
}

pub trait StyleRenderer {
    type Output;
    fn name(&mut self, name: &str);
    fn render(&mut self, rule: &StyleRule);
    fn finish(self) -> Self::Output;
}

#[derive(Debug, Default, Clone)]
pub struct CSSRenderer {
    string: String,
}

impl StyleRenderer for CSSRenderer {
    type Output = String;
    fn name(&mut self, name: &str) {
        write!(self.string, ".{}{{", name).unwrap();
    }
    fn render(&mut self, rule: &StyleRule) {
        write!(self.string, "{}", rule).unwrap()
    }
    fn finish(mut self) -> Self::Output {
        write!(self.string, "}}").unwrap();
        self.string
    }
}

thread_local! {
    static EXPORTS: RefCell<Vec<String>> = {
        println!("");
        Default::default()
    };
}

pub fn export<T: StyleSet>() {
    EXPORTS.with(|exports| {
        let mut renderer = CSSRenderer::default();
        renderer.name(&T::name());
        T::render(&mut renderer);
        exports.borrow_mut().push(renderer.finish());
    });
}

pub fn output() -> String {
    EXPORTS.with(|exports| exports.borrow().join(""))
}

extern crate self as sulafat_style;
