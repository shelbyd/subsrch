use indices::*;

pub struct Range<R> {
    pub passed: Option<Indices>,
    pub failures: Vec<Indices>,
    pub initial_len: usize,
    _strategy: ::std::marker::PhantomData<R>,
}

impl<R> Range<R>
where
    R: RangeStrategy,
{
    pub fn new(initial_len: usize) -> Range<R> {
        Range {
            passed: None,
            failures: Vec::new(),
            initial_len,
            _strategy: ::std::marker::PhantomData,
        }
    }

    pub fn test_passed(&mut self, included_indices: Indices) {
        self.passed = Some(match self.passed.clone() {
            None => included_indices,
            Some(p) => {
                if R::retain_indices(&p, &included_indices) {
                    included_indices
                } else {
                    p
                }
            }
        });
        let passed = self.passed.as_ref().unwrap();
        self.failures.retain(|f| R::retain_indices(passed, f));
    }

    pub fn test_failed(&mut self, indices: Indices) {
        self.failures.push(indices);
    }

    pub fn next(&self) -> RangeNext {
        match (self.passed.as_ref(), self.failures.len()) {
            (Some(passed), _) => R::next_indices(self)
                .map(RunTest)
                .unwrap_or_else(|| Done(Some(passed.clone()))),
            (None, 0) => RunTest(R::initial_test(self)),
            (None, _) => Done(None),
        }
    }
}

pub trait RangeStrategy
where
    Self: Sized,
{
    fn retain_indices(last_passed: &Indices, new_indices: &Indices) -> bool;
    fn initial_test(range: &Range<Self>) -> Indices;
    fn next_indices(range: &Range<Self>) -> Option<Indices>;

    fn new(initial_len: usize) -> Range<Self> {
        Range::<Self>::new(initial_len)
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum RangeNext {
    RunTest(Indices),
    Done(Option<Indices>),
}
pub use self::RangeNext::*;
