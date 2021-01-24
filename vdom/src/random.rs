use crate::cmd::Cmd;
use rand::{
    distributions::uniform::{SampleRange, SampleUniform},
    Rng,
};

pub fn range<T, R, F, Msg>(f: F, range: R) -> Cmd<Msg>
where
    F: 'static + Fn(T) -> Msg,
    T: SampleUniform,
    R: 'static + SampleRange<T>,
{
    Cmd::with(move || f(rand::thread_rng().gen_range(range)))
}
