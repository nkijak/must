
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
    //let args = cli::Args{targets:false, command:String::new()};
    if let Some(cmd) = args.command {
        println!("running {}", cmd);
    }

    let unparsedfile = fs::read_to_string("mustfile").expect("couldnt read mustfile");
    let file = Pestfile::parse(Rule::must, &unparsedfile)
        .expect("unsucessful parse")
        .next().unwrap();

    if args.targets {
        for task in file.into_inner() {
            match task.as_rule() {
                Rule::task => {
                    for target in task.into_inner() {
                        match target.as_rule() {
                            Rule::target => {
                                println!("{}", target.as_str())
                            }
                            _ => println!("") 
                        }
                    }
                }
                _ => println!("")
            }
        }
    }
}


