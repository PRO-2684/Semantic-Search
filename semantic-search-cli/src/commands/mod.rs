//! Subcommands for the Semantic Search CLI.

mod index;
mod search;
mod serve;

use argh::FromArgs;
pub use index::{Index, IndexRecord};

/// Possible commands.
#[derive(FromArgs, PartialEq, Eq, Debug)]
#[argh(subcommand)]
pub enum Command {
    /// An index command.
    Index(index::Index),
    /// A search command.
    Search(search::Search),
    /// A serve command.
    Serve(serve::Serve),
}
