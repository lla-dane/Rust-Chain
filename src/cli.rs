// cli.rs

use std::process::exit;

use clap::builder::Str;
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
                .arg(arg!(<ADDRESS>"'The Address it get balance for'"))
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

        if let Some(_) = matches.subcommand_matches("printchain") {
            let bc = Blockchain::open_blockchain()?;
            for block in bc.iter() {
                println!("ITEM {:#?} \n\n", block);
            }
        }

        if let Some(ref matches ) = matches.subcommand_matches("create") {
            if let Some(address) = matches.get_one::<String>("ADDRESS") {
                let address = String::from(address);
                Blockchain::create_blockchain(address.clone())?;
                println!("RUSTCHAIN CREATED BY {}", address);
            }
        }

        if let Some(ref matches) = matches.subcommand_matches("getbalance") {
            if let Some(address) = matches.get_one::<String>("ADDRESS") {
                let address = String::from(address);
                let bc = Blockchain::open_blockchain()?;
                let utxos = bc.find_utxo(&address);
                let mut balance = 0;
                for utxo in utxos {
                    balance += utxo.value;
                }

                println!("Balance of '{}': {} ", address, balance);
            }
        }

        if let Some(ref matches) = matches.subcommand_matches("send") {
            let sender_addr = if let Some(sender) = matches.get_one::<String>("SENDER") {
                sender
            } else {
                println!("SENDER ADDRESS REQUIRED...!!");
                exit(1)
            };

            let receiver_addr = if let Some(receiver) = matches.get_one::<String>("RECEIVER") {
                receiver
            } else {
                println!("RECEIVER ADDRESS REQUIRED...!!");
                exit(1)
            };

            let amount: i32 = if let Some(amount) = matches.get_one::<String>("AMOUNT") {
                amount.parse()?
            } else {
                println!("AMOUNT TO BE SENT REQUIRED...!!");
                exit(1)
            };

            let mut bc = Blockchain::open_blockchain()?;
            let tx = Transaction::new_transaction(&sender_addr, &receiver_addr, amount, &bc)?;
            bc.add_block(vec![tx])?;
            println!("BLOCK CREATES...!!");
        }

        Ok(())
    }

}
