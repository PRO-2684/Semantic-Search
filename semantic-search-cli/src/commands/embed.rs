//! `embed` subcommand

use argh::FromArgs;

/// generate embeddings for the labels
#[derive(FromArgs, PartialEq, Eq, Debug)]
#[argh(subcommand, name = "embed")]
pub struct Embed {
    // ...
}

impl Embed {
    pub fn execute(&self, key: &str) {
        let _ = self;
        println!("Generating embeddings for labels with key: {}", key);
    }
}

// 1024 floats
