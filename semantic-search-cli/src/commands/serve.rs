//! `serve` subcommand

use argh::FromArgs;

/// start a server to search for files
#[derive(FromArgs, PartialEq, Eq, Debug)]
#[argh(subcommand, name = "serve")]
pub struct Serve {
    // ...
}

impl Serve {
    pub fn execute(&self, port: u16, key: &str) {
        println!("Starting server on port {} with key: {}", port, key);
    }
}
