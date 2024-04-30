mod engine;
mod jsonmodels;

use std::io::BufRead;

use crate::engine::engine_options::HowToDictInArray;
use crate::engine::EngineOptions;
use crate::engine::{Engine, Error};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[arg(long, value_enum)]
    how_to_dict_in_array: Option<HowToDictInArray>,

    #[arg(short, long, default_value_t = false)]
    warnings_as_error: bool,

    /// print original cout to stderr
    #[arg(short, long)]
    tee: bool,
}

fn main() {
    // println!("Hello, world!");

    let cli = Cli::parse();
    // println!("cli = {:?}", cli);

    // exit(0);

    let engine_options = EngineOptions::new();
    let mut engine_options = engine_options.with_verbosity(cli.verbose);

    if let Some(how_to_dict_in_array) = cli.how_to_dict_in_array {
        engine_options = engine_options.with_how_to_dict_in_array(how_to_dict_in_array)
    }

    // println!("engine_options = {:?}", engine_options);

    let mut engine = engine::Engine::new(engine_options);

    loop {
        let mut buffer = String::new();
        let size = std::io::stdin().read_line(&mut buffer);
        match size {
            Ok(size) => {
                // println!("size: '{}'", size);
                // println!("buffer: '{}'", buffer);
                if cli.tee {
                    eprint!("{}", buffer);
                }
                match engine.add_line(&buffer) {
                    Ok(_) => {}
                    Err(e) => {
                        if cli.warnings_as_error == false {
                            eprintln!("Warning: {}", e);
                        } else {
                            panic!("Error: {}", e);
                        }
                    }
                }

                if size == 0 {
                    println!("{}", engine.get_json());
                    break;
                }
            }
            Err(_) => {}
        }
    }
}
