// cli.rs

use clap::{arg, Command};

use crate::blockchain::Blockchain;
use crate::errors::Result;
use crate::transaction::{Transaction};

pub struct Cli {}

impl Cli {
    pub fn new() -> Result<Cli> {
        Ok(Cli {})
    }

    pub fn run(&mut self) -> Result<()> {
        let matches = Command::new("Rust-Chain")
            .version("0.1")
            .author("github.com/lla-dane/Rust-Chain")
            .subcommand(Command::new("printchain").about("print all the chain blocks"))
            .subcommand(Command::new("getbalance")
                .about("get balance in the blockchain")
                .arg(agr!(<ADDRESS>"'The Address it get balance for'"))
            )
            .subcommand(Command::new("create").about("Create new blockchain")
                .arg(arg!(<ADDRESS>"'The address to send genesis block reward to'"))
            )
            .subcommand(
                Command::new("send")
                    .about("send in the blockchain")
                    .arg(arg!(<SENDER>"'Source wallet address'"))
                    .arg(arg!(<RECEIVER>"'Destination wallet address'"))
                    .arg(arg!(<AMOUNT>"'Destination wallet address'")),
            )

            .get_matches();


        
    }

}
