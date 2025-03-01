//! `search` subcommand

use argh::FromArgs;

/// search for files based on labels
#[derive(FromArgs, PartialEq, Eq, Debug)]
#[argh(subcommand, name = "search")]
pub struct Search {
    // ...
}

impl Search {
    pub fn execute(&self, key: &str) {
        println!("Searching for files with key: {}", key);
    }
}
