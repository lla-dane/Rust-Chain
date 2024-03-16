// cli.rs

use std::process::exit;

use bitcoincash_addr::Address;
use clap::{arg, Command};

use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::errors::Result;
use crate::transaction::Transaction;
use crate::utxoset::UTXOSet;
use crate::wallet::Wallets;

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
            .subcommand(Command::new("createwallet").about("create a wallet"))
            .subcommand(Command::new("listaddresses").about("list all addresses"))
            .subcommand(Command::new("reindex").about("reindex UTXOs in the DB"))
            .subcommand(
                Command::new("getbalance")
                    .about("get balance in the blockchain")
                    .arg(arg!(<ADDRESS>"'The Address it get balance for'")),
            )
            .subcommand(
                Command::new("create")
                    .about("Create new blockchain")
                    .arg(arg!(<ADDRESS>"'The address to send genesis block reward to'")),
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

        if let Some(_) = matches.subcommand_matches("createwallet") {
            let mut ws = Wallets::new()?;
            let address = ws.create_wallet();
            ws.save_all()?;
            println!("success: address {}", address);
        }

        if let Some(_) = matches.subcommand_matches("listaddresses") {
            let ws = Wallets::new()?;
            let addresses = ws.get_all_address();
            println!("addresses: ");
            for addr in addresses {
                println!("{}", addr);
            }
        }

        if let Some(_) = matches.subcommand_matches("reindex") {
            let bc = Blockchain::open_blockchain()?;
            let utxo_set = UTXOSet { blockchain: bc };
            utxo_set.reindex()?;
            let count = utxo_set.count_transactions()?;
            println!("Done! There are {} transactions in the UTXO set.", count);
        }

        if let Some(ref matches) = matches.subcommand_matches("create") {
            if let Some(address) = matches.get_one::<String>("ADDRESS") {
                let address = String::from(address);
                let bc = Blockchain::create_blockchain(address.clone())?;
                let utxo_set = UTXOSet { blockchain: bc };
                utxo_set.reindex()?;
                println!("SUCCESS..! BLOCKCHAIN CREATED");
            }
        }

        if let Some(ref matches) = matches.subcommand_matches("getbalance") {
            if let Some(address) = matches.get_one::<String>("ADDRESS") {
                let pub_key_hash = Address::decode(&address).unwrap().body;
                let bc = Blockchain::open_blockchain()?;
                // let utxos = bc.find_utxo(&pub_key_hash);

                let utxo_set = UTXOSet { blockchain: bc };
                let utxos = utxo_set.find_utxo_for_address(&pub_key_hash)?;

                let mut balance = 0;
                for utxo in utxos.outputs {
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

            let bc = Blockchain::open_blockchain()?;
            let mut utxo_set = UTXOSet { blockchain: bc };
            let tx = Transaction::new_transaction(sender_addr, receiver_addr, amount, &utxo_set)?;

            let cbtx = Transaction::new_coinbase(sender_addr.to_string(), String::from("reward"))?;
            let new_block = utxo_set.blockchain.add_block(vec![cbtx, tx])?;

            utxo_set.update(&new_block)?;
            println!("BLOCK CREATED");
        }

        Ok(())
    }
}
