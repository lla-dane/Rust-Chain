// cli.rs

use clap::{arg, Command};

use crate::blockchain::Blockchain;
use crate::errors::Result;
use crate::transaction::Transaction;

pub struct Cli {
    bc: Blockchain,
}

impl Cli {
    pub fn new() -> Result<Cli> {
        Ok(Cli {
            bc: Blockchain::open_blockchain()?,
        })
    }
    pub fn run(&mut self) -> Result<()> {
        let matches = Command::new("Rust-Chain")
            .version("0.1")
            .author("github.com/lla-dane/Rust-Chain")
            .about("Blockchain in Rust: Simple chain for learning")
            .subcommand(Command::new("printchain").about("print all the chain blocks"))
            .subcommand(
                Command::new("addblock")
                    .about("add a block to the blockchain")
                    .arg(arg!(<DATA>"'the blockchain data'")),
            )
            .get_matches();

        if let Some(ref matches) = matches.subcommand_matches("addblock") {
            if let Some(c) = matches.get_one::<String>("DATA") {
                self.add_block(String::from(c))?;
            } else {
                println!("Not printing testing lists...");
            }
        }

        if let Some(_) = matches.subcommand_matches("printchain") {
            self.print_chain();
        }

        Ok(())
    }

    fn add_block(&mut self, data: Vec<Transaction>) -> Result<()> {
        self.bc.add_block(data)
    }

    fn print_chain(&mut self) {
        for b in &mut self.bc.iter() {
            println!("block: {:#?}", b);
        }
    }
}
