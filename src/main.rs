extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use std::collections::*;
use std::iter::*;
use std::io::*;
use std::process::*;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "subsrch", about = "Find subsets that pass the given test.")]
enum SubSrch {
    #[structopt(name = "maximal", about = "Find the largest subset that passes the test.")]
    Maximal {
        #[structopt(help = "Test command.")] test_command: Vec<String>,
    },
    #[structopt(name = "minimal", about = "Find the smallest subset that passes the test.")]
    Minimal {
        #[structopt(help = "Test command.")] test_command: Vec<String>,
    },
}

fn main() {
    let sub_search = SubSrch::from_args();

    let lines = {
        let stdin = stdin();
        let result = BufReader::new(stdin.lock())
            .lines()
            .collect::<Result<Vec<_>>>()
            .unwrap();
        result
    };

    let result = match sub_search {
        SubSrch::Maximal { test_command: c } => {
            let mut command = to_command(c);
            command.stdin(Stdio::piped());

            maximal(lines.clone(), |test_lines| {
                let mut child = command.spawn().unwrap();
                {
                    let child_stdin = child.stdin.as_mut().unwrap();
                    for line in test_lines {
                        child_stdin
                            .write_all((line.clone() + "\n").as_bytes())
                            .unwrap();
                    }
                }
                child.wait().unwrap().success()
            })
        }
        SubSrch::Minimal { test_command: c } => {
            let mut command = to_command(c);
            command.stdin(Stdio::piped());

            minimal(lines, |test_lines| {
                let mut child = command.spawn().unwrap();
                {
                    let child_stdin = child.stdin.as_mut().unwrap();
                    for line in test_lines {
                        child_stdin
                            .write_all((line.clone() + "\n").as_bytes())
                            .unwrap();
                    }
                }
                child.wait().unwrap().success()
            })
        }
    };

    for line in result.unwrap() {
        println!("{}", line);
    }
}

fn to_command(s: Vec<String>) -> Command {
    let mut iter = s.into_iter();
    let cmd = iter.next().unwrap();
    let mut c = Command::new(cmd);
    c.args(&iter.collect::<Vec<_>>());
    c
}

fn maximal<T, F>(full: Vec<T>, mut test: F) -> Option<Vec<T>>
where
    T: Clone,
    F: FnMut(&[T]) -> bool,
{
    let mut reject_range = RejectRange::new(full.len());
    loop {
        match reject_range.next() {
            Done(o) => return o.map(|indices| reject_indices(&full, &indices)),
            RunTest(indices) => {
                if test(&reject_indices(&full, &indices)) {
                    reject_range.test_passed(indices);
                } else {
                    reject_range.test_failed(indices);
                }
            }
        }
    }
}

fn reject_indices<T>(list: &[T], indices: &HashSet<usize>) -> Vec<T>
where
    T: Clone,
{
    list.iter()
        .enumerate()
        .filter(|&(i, _)| !indices.contains(&i))
        .map(|(_, t)| t)
        .cloned()
        .collect()
}

fn minimal<T, F>(full: Vec<T>, mut test: F) -> Option<Vec<T>>
where
    T: Clone,
    F: FnMut(&[T]) -> bool,
{
    let mut reject_range = RejectRange::new(full.len());
    loop {
        match reject_range.next() {
            Done(o) => return o.map(|indices| filter_indices(&full, &indices)),
            RunTest(indices) => {
                if test(&filter_indices(&full, &indices)) {
                    reject_range.test_passed(indices);
                } else {
                    reject_range.test_failed(indices);
                }
            }
        }
    }
}

fn filter_indices<T>(list: &[T], indices: &HashSet<usize>) -> Vec<T>
where
    T: Clone,
{
    list.iter()
        .enumerate()
        .filter(|&(i, _)| indices.contains(&i))
        .map(|(_, t)| t)
        .cloned()
        .collect()
}

macro_rules! set {
    ($($item:expr),*) => {{
        #[allow(unused_mut)]
        let mut s = HashSet::new();
        $(s.insert($item);)*
        s
    }}
}

struct RejectRange {
    passed: Option<HashSet<usize>>,
    failures: Vec<HashSet<usize>>,
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

    fn test_passed(&mut self, included_indices: HashSet<usize>) {
        self.passed = Some(match self.passed.take() {
            None => included_indices,
            Some(p) => {
                if included_indices.is_subset(&p) {
                    included_indices
                } else {
                    p
                }
            }
        });
        let passed = self.passed.as_ref().unwrap();
        self.failures.retain(|f| f.is_subset(passed));
    }

    fn test_failed(&mut self, included_indices: HashSet<usize>) {
        self.failures.push(included_indices);
    }

    fn next(&self) -> RangeNext {
        match (self.passed.as_ref(), self.failures.len()) {
            (Some(passed), _) => self.next_indices()
                .map(RunTest)
                .unwrap_or_else(|| Done(Some(passed.clone()))),
            (None, 0) => RunTest((0..self.initial_len).collect()),
            (None, _) => Done(None),
        }
    }

    fn next_indices(&self) -> Option<HashSet<usize>> {
        let passed = self.passed.as_ref().unwrap();
        let past_rejects: Vec<_> = once(set![])
            .chain(self.failures.iter().map(|f| passed - f))
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

#[derive(PartialEq, Eq, Debug)]
enum RangeNext {
    RunTest(HashSet<usize>),
    Done(Option<HashSet<usize>>),
}
use self::RangeNext::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod minimal {
        use super::*;

        #[test]
        fn empty_passes_is_empty() {
            assert_eq!(minimal(vec![], |_: &[u8]| true), Some(vec![]));
        }

        #[test]
        fn empty_fails_is_none() {
            assert_eq!(minimal(vec![], |_: &[u8]| false), None);
        }

        #[test]
        fn requires_item() {
            assert_eq!(minimal(vec![0], |v| !v.is_empty()), Some(vec![0]));
        }

        #[test]
        fn requires_only_first_item() {
            assert_eq!(minimal(vec![0, 1], |v| v.contains(&0)), Some(vec![0]));
        }

        #[test]
        fn requires_only_second_item() {
            assert_eq!(minimal(vec![0, 1], |v| v.contains(&1)), Some(vec![1]));
        }

        #[test]
        fn requires_both_items() {
            assert_eq!(
                minimal(vec![0, 1], |v| v.contains(&0) && v.contains(&1)),
                Some(vec![0, 1])
            );
        }

        #[test]
        fn requires_first_and_last() {
            assert_eq!(
                minimal(vec![0, 1, 2, 3], |v| v.contains(&0) && v.contains(&3)),
                Some(vec![0, 3])
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
        fn single_element_is_test_all() {
            let range = RejectRange::new(1);

            assert_eq!(range.next(), RunTest(set![0]));
        }

        #[test]
        fn single_element_failed_test_is_none() {
            let mut range = RejectRange::new(1);

            range.test_failed(set![0]);
            assert_eq!(range.next(), Done(None));
        }

        #[test]
        fn single_element_passed_test_is_test_empty() {
            let mut range = RejectRange::new(1);

            range.test_passed(set![0]);
            assert_eq!(range.next(), RunTest(set![]));
        }

        #[test]
        fn single_element_required_is_done() {
            let mut range = RejectRange::new(1);

            range.test_passed(set![0]);
            range.test_failed(set![]);
            assert_eq!(range.next(), Done(Some(set![0])));
        }

        #[test]
        fn empty_passed_with_one_element_is_done() {
            let mut range = RejectRange::new(1);

            range.test_passed(set![]);
            assert_eq!(range.next(), Done(Some(set![])));
        }

        #[test]
        fn two_elements_failed() {
            let mut range = RejectRange::new(2);

            range.test_failed(set![0, 1]);
            assert_eq!(range.next(), Done(None));
        }

        #[test]
        fn two_elements_required() {
            let mut range = RejectRange::new(2);

            range.test_passed(set![0, 1]);
            range.test_failed(set![1]);
            range.test_failed(set![0]);
            assert_eq!(range.next(), Done(Some(set![0, 1])));
        }

        #[test]
        fn two_elements_first_required() {
            let mut range = RejectRange::new(2);

            range.test_passed(set![0, 1]);
            range.test_failed(set![1]);
            assert_eq!(range.next(), RunTest(set![0]));
        }

        #[test]
        fn ignores_larger_passes() {
            let mut range = RejectRange::new(2);

            range.test_passed(set![1]);
            range.test_passed(set![0, 1]);
            assert_eq!(range.next(), RunTest(set![]));
        }

        #[test]
        fn four_elements_requires_middle_two() {
            let mut range = RejectRange::new(4);

            range.test_passed(set![0, 1, 2, 3]);
            range.test_failed(set![0, 1]);
            range.test_failed(set![2, 3]);
            match range.next() {
                RunTest(v) => assert!(v.len() == 3, "{:?} should have length 3", v),
                _ => assert!(false),
            }
        }
    }
}
