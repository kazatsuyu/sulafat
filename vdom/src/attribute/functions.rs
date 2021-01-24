use std::rc::Rc;

use crate::{Attribute, Handler};

use super::style::ToStyle;

pub fn id<Msg>(s: String) -> Attribute<Msg> {
    Attribute::Id(s)
}

pub fn on_click<Msg, F>(f: F) -> Attribute<Msg>
where
    F: 'static + Fn(()) -> Msg,
    Msg: 'static,
{
    Attribute::OnClick(Rc::new(Handler::new(f)))
}

pub fn on_pointer_move<Msg, F>(f: F) -> Attribute<Msg>
where
    F: 'static + Fn((f64, f64)) -> Msg,
    Msg: 'static,
{
    Attribute::OnPointerMove(Rc::new(Handler::new(f)))
}

pub fn style<Msg, S>(s: S) -> Attribute<Msg>
where
    S: ToStyle,
{
    Attribute::Style(s.to_style())
}
