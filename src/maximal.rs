use indices::*;
use std::collections::*;
use std::iter::*;

pub fn maximal<T, F>(full: Vec<T>, mut test: F) -> Option<Vec<T>>
where
    T: Clone,
    F: FnMut(&[T]) -> bool,
{
    let mut reject_range = RejectRange::new(full.len());
    loop {
        match reject_range.next() {
            Done(o) => return o.map(|indices| full.select_indices(&indices)),
            RunTest(indices) => {
                if test(&full.clone().select_indices(&indices)) {
                    reject_range.test_passed(indices);
                } else {
                    reject_range.test_failed(indices);
                }
            }
        }
    }
}

struct RejectRange {
    passed: Option<Indices>,
    failures: Vec<Indices>,
    initial_len: usize,
}

impl RejectRange {
    fn new(list_len: usize) -> RejectRange {
        RejectRange {
            passed: None,
            failures: vec![],
            initial_len: list_len,
        }
    }

    fn test_passed(&mut self, included_indices: Indices) {
        self.passed = Some(match self.passed.take() {
            None => included_indices,
            Some(p) => {
                if included_indices.is_superset(&p) {
                    included_indices
                } else {
                    p
                }
            }
        });
        let passed = self.passed.as_ref().unwrap();
        self.failures.retain(|f| f.is_superset(passed));
    }

    fn test_failed(&mut self, included_indices: Indices) {
        self.failures.push(included_indices);
    }

    fn next(&self) -> RangeNext {
        match (self.passed.as_ref(), self.failures.len()) {
            (Some(passed), _) => self.next_indices()
                .map(RunTest)
                .unwrap_or_else(|| Done(Some(passed.clone()))),
            (None, 0) => RunTest(set![]),
            (None, _) => Done(None),
        }
    }

    fn next_indices(&self) -> Option<Indices> {
        let passed = self.passed.as_ref().unwrap();
        let past_includes: Vec<_> = once(set![])
            .chain(self.failures.iter().map(|f| f - passed))
            .collect();

        let all_elements: HashSet<_> = (0..self.initial_len).collect();
        let unpassed = &all_elements - passed;

        let potential_includes = once(&unpassed)
            .chain(past_includes.iter())
            .flat_map(|include| {
                let mut first: Vec<_> = include.iter().cloned().collect();
                first.sort();
                let mid = first.len() / 2;
                let second = first.split_off(mid);
                vec![first, second]
            })
            .map(|vec| vec.into_iter().collect::<HashSet<_>>());
        potential_includes
            .filter(|i| !past_includes.contains(i))
            .next()
            .map(|next_include| passed | &next_include)
    }
}

#[derive(PartialEq, Eq, Debug)]
enum RangeNext {
    RunTest(Indices),
    Done(Option<Indices>),
}
use self::RangeNext::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod maximal {
        use super::*;

        #[test]
        fn empty_passes_is_empty() {
            assert_eq!(maximal(vec![], |_: &[u8]| true), Some(vec![]));
        }

        #[test]
        fn empty_fails_is_none() {
            assert_eq!(maximal(vec![], |_: &[u8]| false), None);
        }

        #[test]
        fn requires_empty() {
            assert_eq!(maximal(vec![0], |v| v.is_empty()), Some(vec![]));
        }

        #[test]
        fn rejects_first_item() {
            assert_eq!(maximal(vec![0, 1], |v| !v.contains(&0)), Some(vec![1]));
        }

        #[test]
        fn rejects_second_item() {
            assert_eq!(maximal(vec![0, 1], |v| !v.contains(&1)), Some(vec![0]));
        }

        #[test]
        fn two_elements_requires_empty() {
            assert_eq!(
                maximal(vec![0, 1], |v| v.is_empty()),
                Some(vec![])
            );
        }

        #[test]
        fn rejects_first_and_last() {
            assert_eq!(
                maximal(vec![0, 1, 2, 3], |v| !v.contains(&0) && !v.contains(&3)),
                Some(vec![1, 2])
            );
        }
    }

    #[cfg(test)]
    mod reject_range {
        use super::*;

        #[test]
        fn empty_list_should_test_once() {
            let range = RejectRange::new(0);

            assert_eq!(range.next(), RunTest(set![]));
        }

        #[test]
        fn empty_list_failed_test_final_range_is_none() {
            let mut range = RejectRange::new(0);

            range.test_failed(set![]);
            assert_eq!(range.next(), Done(None));
        }

        #[test]
        fn empty_list_passed_test_final_range_is_empty() {
            let mut range = RejectRange::new(0);

            range.test_passed(set![]);
            assert_eq!(range.next(), Done(Some(set![])));
        }

        #[test]
        fn single_element_is_test_empty() {
            let range = RejectRange::new(1);

            assert_eq!(range.next(), RunTest(set![]));
        }

        #[test]
        fn single_element_failed_is_empty_set() {
            let mut range = RejectRange::new(1);

            range.test_passed(set![]);
            range.test_failed(set![0]);
            assert_eq!(range.next(), Done(Some(set![])));
        }

        #[test]
        fn single_element_passed_test_is_done() {
            let mut range = RejectRange::new(1);

            range.test_passed(set![0]);
            assert_eq!(range.next(), Done(Some(set![0])));
        }

        #[test]
        fn two_elements_failed() {
            let mut range = RejectRange::new(2);

            range.test_failed(set![]);
            assert_eq!(range.next(), Done(None));
        }

        #[test]
        fn two_elements_only_one() {
            let mut range = RejectRange::new(2);

            range.test_passed(set![0]);
            range.test_failed(set![0, 1]);
            assert_eq!(range.next(), Done(Some(set![0])));
        }

        #[test]
        fn two_elements_first_required() {
            let mut range = RejectRange::new(2);

            range.test_passed(set![]);
            range.test_failed(set![0, 1]);
            range.test_failed(set![1]);
            assert_eq!(range.next(), RunTest(set![0]));
        }

        #[test]
        fn ignores_smaller_passes() {
            let mut range = RejectRange::new(2);

            range.test_passed(set![0, 1]);
            range.test_passed(set![1]);
            assert_eq!(range.next(), Done(Some(set![0, 1])));
        }

        #[test]
        fn four_elements_rejects_middle_two() {
            let mut range = RejectRange::new(4);

            range.test_passed(set![]);
            range.test_failed(set![0, 1, 2, 3]);
            range.test_failed(set![0, 1]);
            range.test_failed(set![2, 3]);
            match range.next() {
                RunTest(v) => assert!(v.len() == 1, "{:?} should have length 1", v),
                _ => assert!(false),
            }
        }
    }
}
