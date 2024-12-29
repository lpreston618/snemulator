use std::process::exit;

use getopts::Occur;
use snemulator::Config;

extern crate args;
extern crate getopts;

use args::{Args,ArgsError};

const PROGRAM_DESC: &'static str = "Run the snemulator";
const PROGRAM_NAME: &'static str = "snemulator";


fn main() {
    match parse(std::env::args().collect()) {
        Ok(config) => {
            println!("Successfully parsed args");
            snemulator::run(config);
        },
        Err(error) => {
            println!("{}", error);
            exit(1);
        }
    };
}


fn parse(input: Vec<String>) -> Result<Config, ArgsError> {
    let mut args = Args::new(&PROGRAM_NAME, &PROGRAM_DESC);
    args.flag("h", "help", "Print the usage menu");
    args.option("p", 
        "path", 
        "Path of the ROM to run", 
        "PATH", 
        Occur::Req, 
        None);

    if let Err(err) = args.parse(input) {
        println!("{}", args.full_usage());
        return Err(err);
    }

    if args.value_of("help").unwrap() {
        println!("{}", args.full_usage());
    }

    return Ok(Config {
        rom_path: args.value_of("path").unwrap(),
    });
}