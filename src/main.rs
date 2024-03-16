// main.rs

mod block;
mod blockchain;
mod cli;
mod errors;
mod transaction;
mod tx;
mod wallet;
mod utxoset;

use crate::cli::Cli;
use crate::errors::Result;

fn main() -> Result<()> {
    let mut cli = Cli::new()?;
    cli.run()?;
    Ok(())
}
