use std::env;
use std::process;

use saver::Saver;

fn main() {
    let saver = Saver::build(env::args());
    if let Err(e) = saver.run() {
        println!("{}", e);
        process::exit(1);
    }
}
