extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "subsrch", about = "Find subsets that pass the given test.")]
enum SubSrch {
    #[structopt(name = "maximal", about = "Find the largest subset that passes the test.")]
    Maximal {
        #[structopt(help = "Test command.")] test_command: String,
    },
}

fn main() {
    let sub_search = SubSrch::from_args();
    match sub_search {
        SubSrch::Maximal { test_command: c } => {
            println!("{}", c);
        }
    }
}
