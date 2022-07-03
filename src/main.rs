
use clap::Parser as ClapParser;
extern crate pest;
#[macro_use]
extern crate pest_derive;
use pest::Parser;
use std::fs;

mod cli;


#[derive(Parser)]
#[grammar = "mustfile.pest"]
pub struct Pestfile;



fn main() {
    let args = cli::Args::parse();
    if let Some(cmd) = args.command {
        println!("running {}", cmd);
    }

    let unparsedfile = fs::read_to_string("mustfile").expect("couldnt read mustfile");
    let mustfile = Pestfile::parse(Rule::must, &unparsedfile)
        .expect("unsucessful parse");

    if args.targets {
        for entry in mustfile {
            match entry.as_rule() {
                Rule::task => {
                    println!("{}", entry.into_inner().next().unwrap().as_str());
                }
                _ => ()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_target() {
        let testfile = fs::read_to_string("tests/fixtures/single.must").expect("couldnt find test file");
        let mut file = Pestfile::parse(Rule::must, &testfile)
            .expect("unscuessful parse");

        let task = file.next().unwrap();
        assert_eq!(task.as_rule(), Rule::task);
        assert_eq!(file.next().expect("premature end").as_rule(), Rule::EOI)
    }

    #[test]
    fn parses_multiple_targets() {
        let testfile = fs::read_to_string("tests/fixtures/single.must").expect("couldnt find test file");
        let mut file = Pestfile::parse(Rule::must, &testfile)
            .expect("unscuessful parse");

        let task = file.next().unwrap();
        assert_eq!(task.as_rule(), Rule::task);
        println!("{}", task.as_str());
        assert_eq!(file.next().expect("task not found").as_rule(), Rule::task);
        assert!(file.next().is_none())
    }
}
