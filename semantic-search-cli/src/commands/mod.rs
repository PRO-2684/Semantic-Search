//! Subcommands for the Semantic Search CLI.

mod embed;
mod index;
mod search;
mod serve;

use argh::FromArgs;
pub use index::{index, IndexRecord};

/// Possible commands.
#[derive(FromArgs, PartialEq, Eq, Debug)]
#[argh(subcommand)]
pub enum Command {
    /// An index command.
    Index(index::Index),
    /// An embed command.
    Embed(embed::Embed),
    /// A search command.
    Search(search::Search),
    /// A serve command.
    Serve(serve::Serve),
}
