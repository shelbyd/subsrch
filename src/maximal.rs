use indices::*;
use range::*;
use std::collections::*;
use std::iter::*;

pub struct MaximalRange;

impl RangeStrategy for MaximalRange {
    fn retain_indices(passed: &Indices, new_indices: &Indices) -> bool {
        new_indices.is_superset(passed)
    }

    fn initial_test(_: &Range<Self>) -> Indices {
        set![]
    }

    // TODO(shelbyd): Reduce duplication in this method.
    fn next_indices(range: &Range<Self>) -> Option<Indices> {
        let passed = range.passed.as_ref().unwrap();
        let past_includes: Vec<_> = once(set![])
            .chain(range.failures.iter().map(|f| f - passed))
            .collect();

        let all_elements: HashSet<_> = (0..range.initial_len).collect();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_list_should_test_once() {
        let range = MaximalRange::new(0);

        assert_eq!(range.next(), RunTest(set![]));
    }

    #[test]
    fn empty_list_failed_test_final_range_is_none() {
        let mut range = MaximalRange::new(0);

        range.test_failed(set![]);
        assert_eq!(range.next(), Done(None));
    }

    #[test]
    fn empty_list_passed_test_final_range_is_empty() {
        let mut range = MaximalRange::new(0);

        range.test_passed(set![]);
        assert_eq!(range.next(), Done(Some(set![])));
    }

    #[test]
    fn single_element_is_test_empty() {
        let range = MaximalRange::new(1);

        assert_eq!(range.next(), RunTest(set![]));
    }

    #[test]
    fn single_element_failed_is_empty_set() {
        let mut range = MaximalRange::new(1);

        range.test_passed(set![]);
        range.test_failed(set![0]);
        assert_eq!(range.next(), Done(Some(set![])));
    }

    #[test]
    fn single_element_passed_test_is_done() {
        let mut range = MaximalRange::new(1);

        range.test_passed(set![0]);
        assert_eq!(range.next(), Done(Some(set![0])));
    }

    #[test]
    fn two_elements_failed() {
        let mut range = MaximalRange::new(2);

        range.test_failed(set![]);
        assert_eq!(range.next(), Done(None));
    }

    #[test]
    fn two_elements_only_one() {
        let mut range = MaximalRange::new(2);

        range.test_passed(set![0]);
        range.test_failed(set![0, 1]);
        assert_eq!(range.next(), Done(Some(set![0])));
    }

    #[test]
    fn two_elements_first_required() {
        let mut range = MaximalRange::new(2);

        range.test_passed(set![]);
        range.test_failed(set![0, 1]);
        range.test_failed(set![1]);
        assert_eq!(range.next(), RunTest(set![0]));
    }

    #[test]
    fn ignores_smaller_passes() {
        let mut range = MaximalRange::new(2);

        range.test_passed(set![0, 1]);
        range.test_passed(set![1]);
        assert_eq!(range.next(), Done(Some(set![0, 1])));
    }

    #[test]
    fn four_elements_rejects_middle_two() {
        let mut range = MaximalRange::new(4);

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
