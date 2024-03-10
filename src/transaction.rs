// transaction.rs


use crypto::{digest::Digest, sha2::Sha256};
use failure::format_err;
use log::error;
use serde::{Deserialize, Serialize};

use crate::errors::Result;
use crate::blockchain::Blockchain;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub id: String, // Transaction ID of the transaction
    pub v_inputs: Vec<TXInput>,
    pub v_outputs: Vec<TXOutput>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXInput {
    pub txid: String, // Transaction ID of the prev transaction from where the input came from.
    pub output_index: i32,    // Index of the output in the previous transaction
    pub script_sig: String, // Signature Script
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXOutput {
    pub value: i32,             // The amount of crypto the that the output holds
    pub script_pub_key: String, // PubKey Script
}

impl Transaction {

    pub fn new_transaction(sender_address: &str, receiver_address: &str, amount:i32, bc: &Blockchain) -> Result<Transaction> {
        let mut v_inputs = Vec::new();
        let balance_utxos = bc.find_spendable_outputs(sender_address, amount);

        // Check if there is enough money to spend
        if balance_utxos.0 < amount {
            error!("Not Enough Balance");
            return Err(format_err!("NOT ENOUGH BALANCE: CURRENT BALANCE {}", balance_utxos.0));
        }

        // creates the inputs list of the transaction
        for txid_outputIndex in balance_utxos.1 {
            for outputIndex in txid_outputIndex.1 {
                let input = TXInput {
                    txid: txid_outputIndex.0.clone(),
                    output_index: outputIndex,
                    script_sig: String::from(sender_address),
                };
                v_inputs.push(input);
            }
        }

        let mut v_outputs = vec![TXOutput {
            value: amount,
            script_pub_key: String::from(receiver_address),
        }];

        if balance_utxos.0 > amount {
            v_outputs.push(TXOutput {
                value: balance_utxos.0 - amount,
                script_pub_key: String::from(sender_address),
            })
        }

        let mut tx = Transaction {
            id: String::new(),
            v_inputs: v_inputs,
            v_outputs: v_outputs,
        };
        
        tx.set_id();
        Ok(tx)
    }

    pub fn new_coinbase(receiver: String, mut data: String) -> Result<Transaction> {
        if data == String::from("") {
            data += &format!("Reward to '{}'", receiver);
        }

        let mut tx = Transaction {
            id: String::new(),
            v_inputs: vec![TXInput {
                txid: String::new(),
                output_index: -1,
                script_sig: data,
            }],
            v_outputs: vec![TXOutput {
                value: 100,
                script_pub_key: receiver,
            }],
        };
        tx.set_id()?;
        Ok(tx)
    }

    // Sets the ID of a transaction
    fn set_id(&mut self) -> Result<()> {
        let mut hasher = Sha256::new();
        let data = bincode::serialize(self)?;
        hasher.input(&data);
        self.id = hasher.result_str();

        Ok(())
    }

    // Check whether the transaction is coinbase
    pub fn is_coinbase(&self) -> bool {
        self.v_inputs.len() == 1 && self.v_inputs[0].txid.is_empty() && self.v_inputs[0].output_index == -1
    }
}

impl TXInput {
    // CanUnlockOutputWith checks whether the address initiated the transaction
    pub fn can_unlock_output_with(&self, sender_address: &str) -> bool {
        self.script_sig == sender_address
    }
}

impl TXOutput {
    // CanBeUnlockedWith checks if the output can be unlocked with the provided data
    pub fn can_be_unlocked_with(&self, receiver_address: &str) -> bool {
        self.script_pub_key == receiver_address
    }
}
