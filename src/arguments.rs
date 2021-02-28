//!
//! Solidity to SyncVM compiler arguments.
//!

use std::path::PathBuf;
use structopt::StructOpt;

///
/// Solidity to SyncVM compiler arguments.
///

#[derive(Debug, StructOpt)]
#[structopt(name = "Solidity compiler for SyncVM")]
pub struct Arguments {
    /// Input file
    #[structopt(parse(from_os_str))]
    pub input: PathBuf,

    /// Function to run
    #[structopt(short, long, default_value = "")]
    pub run: String,

    /// Options to pass to solidity compiler
    #[structopt(long = "Xsol", default_value = "")]
    pub xsol: String,
}

impl Arguments {
    ///
    /// A shortcut constructor.
    ///
    pub fn new() -> Self {
        Self::from_args()
    }
}

impl Default for Arguments {
    fn default() -> Self {
        Self::new()
    }
}
