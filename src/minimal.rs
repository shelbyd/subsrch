use indices::*;
use range::*;
use std::collections::*;
use std::iter::*;

pub struct MinimalRange;

impl RangeStrategy for MinimalRange {
    fn retain_indices(passed: &Indices, new_indices: &Indices) -> bool {
        new_indices.is_subset(passed)
    }

    fn initial_test(range: &Range<Self>) -> Indices {
        (0..range.initial_len).collect()
    }

    // TODO(shelbyd): Reduce duplication in this method.
    fn next_indices(range: &Range<Self>) -> Option<Indices> {
        let passed = range.passed.as_ref().unwrap();
        let past_rejects: Vec<_> = once(set![])
            .chain(range.failures.iter().map(|f| passed - f))
            .collect();
        let potential_rejects = once(passed)
            .chain(past_rejects.iter())
            .flat_map(|reject| {
                let mut first: Vec<_> = reject.iter().cloned().collect();
                first.sort();
                let mid = first.len() / 2;
                let second = first.split_off(mid);
                vec![first, second]
            })
            .map(|vec| vec.into_iter().collect::<HashSet<_>>());
        potential_rejects
            .filter(|r| !past_rejects.contains(r))
            .next()
            .map(|next_reject| passed - &next_reject)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_list_should_test_once() {
        let range = MinimalRange::new(0);

        assert_eq!(range.next(), RunTest(set![]));
    }

    #[test]
    fn empty_list_failed_test_final_range_is_none() {
        let mut range = MinimalRange::new(0);

        range.test_failed(set![]);
        assert_eq!(range.next(), Done(None));
    }

    #[test]
    fn empty_list_passed_test_final_range_is_empty() {
        let mut range = MinimalRange::new(0);

        range.test_passed(set![]);
        assert_eq!(range.next(), Done(Some(set![])));
    }

    #[test]
    fn single_element_is_test_all() {
        let range = MinimalRange::new(1);

        assert_eq!(range.next(), RunTest(set![0]));
    }

    #[test]
    fn single_element_failed_test_is_none() {
        let mut range = MinimalRange::new(1);

        range.test_failed(set![0]);
        assert_eq!(range.next(), Done(None));
    }

    #[test]
    fn single_element_passed_test_is_test_empty() {
        let mut range = MinimalRange::new(1);

        range.test_passed(set![0]);
        assert_eq!(range.next(), RunTest(set![]));
    }

    #[test]
    fn single_element_required_is_done() {
        let mut range = MinimalRange::new(1);

        range.test_passed(set![0]);
        range.test_failed(set![]);
        assert_eq!(range.next(), Done(Some(set![0])));
    }

    #[test]
    fn empty_passed_with_one_element_is_done() {
        let mut range = MinimalRange::new(1);

        range.test_passed(set![]);
        assert_eq!(range.next(), Done(Some(set![])));
    }

    #[test]
    fn two_elements_failed() {
        let mut range = MinimalRange::new(2);

        range.test_failed(set![0, 1]);
        assert_eq!(range.next(), Done(None));
    }

    #[test]
    fn two_elements_required() {
        let mut range = MinimalRange::new(2);

        range.test_passed(set![0, 1]);
        range.test_failed(set![1]);
        range.test_failed(set![0]);
        assert_eq!(range.next(), Done(Some(set![0, 1])));
    }

    #[test]
    fn two_elements_first_required() {
        let mut range = MinimalRange::new(2);

        range.test_passed(set![0, 1]);
        range.test_failed(set![1]);
        assert_eq!(range.next(), RunTest(set![0]));
    }

    #[test]
    fn ignores_larger_passes() {
        let mut range = MinimalRange::new(2);

        range.test_passed(set![1]);
        range.test_passed(set![0, 1]);
        assert_eq!(range.next(), RunTest(set![]));
    }

    #[test]
    fn four_elements_requires_middle_two() {
        let mut range = MinimalRange::new(4);

        range.test_passed(set![0, 1, 2, 3]);
        range.test_failed(set![0, 1]);
        range.test_failed(set![2, 3]);
        match range.next() {
            RunTest(v) => assert!(v.len() == 3, "{:?} should have length 3", v),
            _ => assert!(false),
        }
    }
}
