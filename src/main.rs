use std::env::current_dir;
use std::process;

use jm::run;

fn main() {
    let cwd = current_dir().unwrap();

    match run(&cwd) {
        Ok(()) => println!("Done."),
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    }
}
