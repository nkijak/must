//use pest;
use clap::Parser;

mod cli;


//#[derive(pest::Parser)]
//#[grammar = "mustfile.pest"]
//struct Pestfile;



fn main() {
    let args = cli::Args::parse();
    if args.targets {
        println!("no targets")
    }
    println!("running {}", args.command)
}


