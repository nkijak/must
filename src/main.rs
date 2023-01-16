use std::process::Command;
use std::io::{self, Write};
use clap::Parser as ClapParser;
extern crate pest;
#[macro_use]
extern crate pest_derive;
use pest::Parser;
use pest::iterators::Pairs;
use std::fs;
use std::path;

mod cli;


#[derive(Parser)]
#[grammar = "mustfile.pest"]
pub struct Pestfile;

#[derive(Debug,Default)]
struct Task {
    target: String,
    _deps: Vec<String>,
    dependencies: Vec<Box<Task>>,
    steps: Vec<String>
}

// TODO how to tell it to use the variant `task`?
// TODO this needs to return the Result type
fn execute_task(task: Task) {
    for step in task.steps {
        println!("{}", step);
        let result = Command::new("sh")
            .arg("-c")
            .arg(step)
            .output()
            .expect("Could not execute");
        io::stdout().write_all(&result.stdout).unwrap();
        io::stderr().write_all(&result.stderr).unwrap();
        assert!(result.status.success());
    }
}

fn build_task(taskentries: Pairs<Rule>) -> Task {
    let mut task = Task{target: String::from(""), _deps: vec![], dependencies:vec![], steps:vec![]};
    for entry in taskentries {
        match entry.as_rule() {
            Rule::target    => task.target=entry.as_str().into(),
            Rule::dependent => task._deps.push(entry.into_inner().next().unwrap().as_str().into()),
            Rule::action    => task.steps.push(entry.as_str().into()),
            _ => unreachable!()
        }
    }
    task
}

fn main() {
    let args = cli::Args::parse();

    let filename: String = (if path::Path::new("mustfile").exists() {
        "mustfile"
    } else if path::Path::new("Makefile").exists() {
        "Makefile"
    } else if path::Path::new("makefile").exists() {
        "makefile"
    } else {
        panic!("no mustfile, Makefile, or makefile"); 
    }).into();
    let unparsedfile = fs::read_to_string(filename).expect("couldnt read");
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
    } else {
        let mut tasks = mustfile.map_while (|entry| -> Option<Task> {
            match entry.as_rule() {
                Rule::task => {
                    let task = entry.into_inner();
                    Some(build_task(task))
                },
                Rule::EOI => None,
                _ => unreachable!()
            }
        });

        let task = match &args.command {
            Some(command) => tasks.find(|t| &t.target == command),
            None => tasks.next()
        };
        if task.is_some() {
            execute_task(task.unwrap())
        } else {
            // TODO need to return error code 2 here
            println!("must: *** No rule to make target `{}'. Stop", args.command.as_ref().unwrap())
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
            .expect("unsuccessful parse");

        let task = file.next().unwrap();
        assert_eq!(task.as_rule(), Rule::task);
        assert_eq!(file.next().expect("premature end").as_rule(), Rule::EOI)
    }

    #[test]
    fn parses_multiple_targets() {
        let testfile = fs::read_to_string("tests/fixtures/multiple.must").expect("couldnt find test file");
        let mut file = Pestfile::parse(Rule::must, &testfile)
            .expect("unsuccessful parse");

        let task = file.next().unwrap();
        assert_eq!(task.as_rule(), Rule::task);
        assert_eq!(file.next().expect("task not found").as_rule(), Rule::task);
        assert_eq!(file.next().unwrap().as_rule(), Rule::EOI);
        assert!(file.next().is_none());
    }

    #[test]
    fn build_task_works() {
        let testfile = fs::read_to_string("tests/fixtures/multiple.must").expect("couldnt find test file");
        let mut file = Pestfile::parse(Rule::must, &testfile)
            .expect("unsuccessful parse");

        // target1: dep1
            // step one
            // step two
        let task = file.next().unwrap();
        let actual = build_task(task.into_inner());
        assert_eq!(actual.target, String::from("target1"));
        assert_eq!(actual._deps, vec![String::from("dep1")]);
        assert_eq!(actual.steps, vec![String::from("step one"), String::from("step two")]);
    }
}
