use std::{env, process};

use grepr::Config;

fn main() {
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(1)
    });

    grepr::run(config)
}
