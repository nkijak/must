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

#[derive(Debug,Default,Clone)]
struct Task {
    target: String,
    _deps: Vec<String>,
    dependencies: Vec<Box<Task>>,
    steps: Vec<String>
}

fn execute_task(task: Task, all_tasks: &Vec<Task>) -> Result<(), String> {
    // First execute all dependencies
    for dep_name in &task._deps {
        let dep_task = all_tasks.iter()
            .find(|t| &t.target == dep_name)
            .ok_or_else(|| format!("Dependency '{}' not found", dep_name))?;

        println!("Executing dependency: {}", dep_name);
        execute_task(dep_task.clone(), all_tasks)?;
    }

    // Then execute the main task steps
    for step in task.steps {
        println!("{}", step);
        let result = Command::new("sh")
            .arg("-c")
            .arg(step)
            .output()
            .map_err(|e| format!("Failed to execute command: {}", e))?;

        io::stdout().write_all(&result.stdout).unwrap();
        io::stderr().write_all(&result.stderr).unwrap();

        if !result.status.success() {
            return Err(format!(
                "Command failed with exit code: {}",
                result.status.code().unwrap_or(-1)
            ));
        }
    }
    Ok(())
}

fn build_task(taskentries: Pairs<Rule>) -> Task {
    let mut task = Task{target: String::from(""), _deps: vec![], dependencies:vec![], steps:vec![]};
    for entry in taskentries {
        match entry.as_rule() {
            Rule::target    => task.target=entry.as_str().into(),
            Rule::dependent => task._deps.push(entry.into_inner().next().unwrap().as_str().into()),
            Rule::action    => task.steps.push(entry.as_str().trim().into()),
            _ => unreachable!()
        }
    }
    task
}

/// .
///
/// # Panics
///
/// Panics if .
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
        let tasks: Vec<Task> = mustfile.map_while(|entry| -> Option<Task> {
            match entry.as_rule() {
                Rule::task => {
                    let task = entry.into_inner();
                    Some(build_task(task))
                },
                Rule::EOI => None,
                _ => unreachable!()
            }
        }).collect();

        let task = match &args.command {
            Some(command) => tasks.iter().find(|t| &t.target == command),
            None => tasks.first()
        };

        if let Some(task) = task {
            if let Err(e) = execute_task(task.clone(), &tasks) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        } else {
            println!("must: *** No rule to make target `{}'. Stop", args.command.as_ref().unwrap());
            std::process::exit(2);
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

    #[test]
    fn failing_command_returns_error() {
        let testfile = fs::read_to_string("tests/fixtures/failing_command.must").expect("couldnt find test file");
        let mut file = Pestfile::parse(Rule::must, &testfile)
            .expect("unsuccessful parse");

        let task = file.next().unwrap();
        let task = build_task(task.into_inner());
        let result = execute_task(task, &vec![]);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("Command failed with exit code: 1"));
    }

    #[test]
    fn executes_dependencies_in_order() {
        let testfile = fs::read_to_string("tests/fixtures/dependencies.must").expect("couldnt find test file");
        let mut file = Pestfile::parse(Rule::must, &testfile)
            .expect("unsuccessful parse");

        let tasks: Vec<Task> = file.map_while(|entry| -> Option<Task> {
            match entry.as_rule() {
                Rule::task => {
                    let task = entry.into_inner();
                    Some(build_task(task))
                },
                Rule::EOI => None,
                _ => unreachable!()
            }
        }).collect();

        let main_task = tasks.iter().find(|t| t.target == "main").unwrap();
        let result = execute_task(main_task.clone(), &tasks);
        assert!(result.is_ok());
    }

    #[test]
    fn parses_comments() {
        let testfile = fs::read_to_string("tests/fixtures/comments.must").expect("couldnt find test file");
        let mut file = Pestfile::parse(Rule::must, &testfile)
            .expect("unsuccessful parse");

        let tasks: Vec<Task> = file.map_while(|entry| -> Option<Task> {
            match entry.as_rule() {
                Rule::task => {
                    let task = entry.into_inner();
                    Some(build_task(task))
                },
                Rule::EOI => None,
                _ => unreachable!()
            }
        }).collect();

        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].target, "target1");
        assert_eq!(tasks[0]._deps, vec!["dep1".to_string()]);
        assert_eq!(tasks[0].steps, vec!["echo \"hello world\"".to_string(), "echo hello world".to_string()]);
        assert_eq!(tasks[1].target, "target2");
        assert_eq!(tasks[1]._deps, Vec::<String>::new());
        assert_eq!(tasks[1].steps, vec!["echo \"test\"".to_string(), "echo test".to_string()]);
    }
}
