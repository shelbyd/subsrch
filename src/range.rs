use indices::*;

pub trait Range
where
    Self: Sized,
{
    fn new(list_len: usize) -> Self;
    fn next(&self) -> RangeNext;
    fn test_failed(&mut self, indices: Indices);
    fn test_passed(&mut self, indices: Indices);
}

#[derive(PartialEq, Eq, Debug)]
pub enum RangeNext {
    RunTest(Indices),
    Done(Option<Indices>),
}
pub use self::RangeNext::*;
