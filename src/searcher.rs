use indices::*;
use range::*;
use std::io::*;
use std::process::*;

pub struct Searcher {
    command: Command,
}

impl Searcher {
    pub fn from_str(s: Vec<String>) -> Searcher {
        let mut iter = s.into_iter();
        let cmd = iter.next().unwrap();

        let mut c = Command::new(cmd);
        c.args(&iter.collect::<Vec<_>>());
        c.stdin(Stdio::piped());

        Searcher { command: c }
    }

    pub fn test(&mut self, test_lines: &[String]) -> bool {
        let mut child = self.command.spawn().expect("could not spawn child process");
        {
            let mut child_stdin = child.stdin.take().unwrap();
            for line in test_lines {
                child_stdin
                    .write_all((line.clone() + "\n").as_bytes())
                    .expect("failed to write to process stdin");
            }
        }
        child.wait().expect("process did not finish").success()
    }

    pub fn search<R>(&mut self, full: Vec<String>) -> Option<Vec<String>>
    where
        R: RangeStrategy,
    {
        let mut range = Range::<R>::new(full.len());
        loop {
            match range.next() {
                Done(o) => return o.map(|indices| full.select_indices(&indices)),
                RunTest(indices) => {
                    if self.test(&full.clone().select_indices(&indices)) {
                        range.test_passed(indices);
                    } else {
                        range.test_failed(indices);
                    }
                }
            }
        }
    }
}
