extern crate structopt;
#[macro_use]
extern crate structopt_derive;

macro_rules! set {
    ($($item:expr),*) => {{
        #[allow(unused_mut)]
        let mut s = HashSet::new();
        $(s.insert($item);)*
        s
    }}
}

mod indices;

mod maximal;
use self::maximal::*;

mod minimal;
use self::minimal::*;

mod range;

mod searcher;
use self::searcher::*;

use std::iter::*;
use std::io::*;
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
            .expect("failed to read from stdin");
        result
    };

    let result = match sub_search {
        SubSrch::Maximal { test_command: c } => {
            Searcher::from_str(c).search::<MaximalRange>(lines)
        }
        SubSrch::Minimal { test_command: c } => {
            Searcher::from_str(c).search::<MinimalRange>(lines)
        }
    };

    for line in result.expect("no viable subsets found") {
        println!("{}", line);
    }
}
