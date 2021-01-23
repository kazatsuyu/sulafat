use base58::ToBase58;
use serde_derive::{Deserialize, Serialize};
use std::{cell::RefCell, fmt::Write};
use sulafat_util::TypeId;

// やりたいこと
// * CSSを自動で出力（Web)
// * テーマによる動的なスタイル切り替え
//

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Length {
    Em(f64),
    Px(f64),
    Vh(f64),
    Vw(f64),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parcentage(pub f64);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LengthOrPercentage {
    Length(Length),
    Parcentage(Parcentage),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StyleRule {
    Left(LengthOrPercentage),
    Right(LengthOrPercentage),
}

pub trait StyleSet: 'static {
    fn rules() -> &'static [StyleRule];
    fn id() -> TypeId {
        TypeId::of::<Self>()
    }
    fn name() -> String {
        format!("sulafat-{}", Self::id().inner().to_le_bytes().to_base58())
    }
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
        match rule {
            StyleRule::Left(LengthOrPercentage::Length(Length::Em(em))) => {
                write!(self.string, "left:{}em;", em)
            }
            StyleRule::Left(LengthOrPercentage::Length(Length::Px(px))) => {
                write!(self.string, "left:{}px;", px)
            }
            StyleRule::Left(LengthOrPercentage::Length(Length::Vh(vh))) => {
                write!(self.string, "left:{}vh;", vh)
            }
            StyleRule::Left(LengthOrPercentage::Length(Length::Vw(vw))) => {
                write!(self.string, "left:{}vw;", vw)
            }
            StyleRule::Left(LengthOrPercentage::Parcentage(Parcentage(parcentage))) => {
                write!(self.string, "left:{}%;", parcentage)
            }
            StyleRule::Right(LengthOrPercentage::Length(Length::Em(em))) => {
                write!(self.string, "right:{}em;", em)
            }
            StyleRule::Right(LengthOrPercentage::Length(Length::Px(px))) => {
                write!(self.string, "right:{}px;", px)
            }
            StyleRule::Right(LengthOrPercentage::Length(Length::Vh(vh))) => {
                write!(self.string, "right:{}vh;", vh)
            }
            StyleRule::Right(LengthOrPercentage::Length(Length::Vw(vw))) => {
                write!(self.string, "right:{}vw;", vw)
            }
            StyleRule::Right(LengthOrPercentage::Parcentage(Parcentage(parcentage))) => {
                write!(self.string, "right:{}%;", parcentage)
            }
        }
        .unwrap()
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

#[cfg(test)]
mod test {

    use super::{
        CSSRenderer, Length, LengthOrPercentage, Parcentage, StyleRenderer, StyleRule, StyleSet,
    };
    use sulafat_macros::StyleSet;

    #[derive(StyleSet)]
    #[style_set{
        .test {
            left: 100px;
            right: 100%;
        }
    }]
    struct Style;

    #[test]
    fn it_works() {
        assert_eq!(
            Style::rules(),
            &[
                StyleRule::Left(LengthOrPercentage::Length(Length::Px(100.))),
                StyleRule::Right(LengthOrPercentage::Parcentage(Parcentage(100.))),
            ]
        )
    }

    #[test]
    fn css_renderer() {
        let mut renderer = CSSRenderer::default();
        renderer.name(&Style::name());
        Style::render(&mut renderer);
        assert_eq!(renderer.finish(), ".test{left:100px;right:100%;}");
    }
}
