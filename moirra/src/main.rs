use quicli::prelude::*;
use structopt::StructOpt;

#[derive(Debug,StructOpt)]
struct Cli {
    #[structopt(long = "config", short = "c")]
    config: Option<String>,
}

fn main() {
    let args = Cli::from_args();

    match args.config {
        Some(_) => println!("lajsdf;"),
        _ => println!("no match"),
    }

    println!("Hello, world!");
}


