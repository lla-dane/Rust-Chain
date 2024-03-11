// transaction.rs

use std::collections::HashMap;

use crypto::ed25519;
use crypto::{digest::Digest, sha2::Sha256};
use failure::format_err;
use log::error;
use serde::{Deserialize, Serialize};

use crate::blockchain::Blockchain;
use crate::errors::Result;
use crate::tx::{TXInput, TXOutput};
use crate::wallet::{hash_pub_key, Wallets};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub id: String, // Transaction ID of the transaction
    pub v_inputs: Vec<TXInput>,
    pub v_outputs: Vec<TXOutput>,
}

impl Transaction {
    // Creates a new transaction
    pub fn new_transaction(
        sender_address: &str,
        receiver_address: &str,
        amount: i32,
        bc: &Blockchain,
    ) -> Result<Transaction> {
        let mut v_inputs = Vec::new();

        let wallets = Wallets::new()?;
        let wallet = match wallets.get_wallet(sender_address) {
            Some(wallet) => wallet,
            None => return Err(format_err!("Sender wallet not found")),
        };

        if let None = wallets.get_wallet(&receiver_address) {
            return Err(format_err!("Receiver wallet not found"));
        };

        let mut pub_key_hash = wallet.public_key.clone();
        hash_pub_key(&mut pub_key_hash);

        let balance_utxos = bc.find_spendable_outputs(&pub_key_hash, amount);
        // Check if there is enough money to spend
        if balance_utxos.0 < amount {
            error!("Not Enough Balance");
            return Err(format_err!(
                "NOT ENOUGH BALANCE: CURRENT BALANCE {}",
                balance_utxos.0
            ));
        }

        // creates the inputs list of the transaction
        for txid_output_index in balance_utxos.1 {
            for output_index in txid_output_index.1 {
                let input = TXInput {
                    txid: txid_output_index.0.clone(),
                    output_index: output_index,
                    signature: Vec::new(),
                    pub_key: wallet.public_key.clone(),
                };
                v_inputs.push(input);
            }
        }

        let mut v_outputs = vec![TXOutput::new(amount, receiver_address.to_string())?];

        if balance_utxos.0 > amount {
            v_outputs.push(TXOutput::new(
                balance_utxos.0 - amount,
                sender_address.to_string(),
            )?)
        }

        let mut tx = Transaction {
            id: String::new(),
            v_inputs: v_inputs,
            v_outputs: v_outputs,
        };

        tx.id = tx.hash()?;
        bc.sign_transaction(&mut tx, &wallet.private_key)?;

        Ok(tx)
    }

    // Creates a new COINBASE TRANSACTION with the miner's address
    pub fn new_coinbase(receiver: String, mut data: String) -> Result<Transaction> {
        if data == String::from("") {
            data += &format!("Reward to '{}'", receiver);
        }

        let mut tx = Transaction {
            id: String::new(),
            v_inputs: vec![TXInput {
                txid: String::new(),
                output_index: -1,
                signature: Vec::new(),
                pub_key: Vec::from(data.as_bytes()),
            }],
            v_outputs: vec![TXOutput::new(100, receiver)?],
        };
        tx.id = tx.hash()?;
        Ok(tx)
    }

    // Check whether the transaction is coinbase
    pub fn is_coinbase(&self) -> bool {
        self.v_inputs.len() == 1
            && self.v_inputs[0].txid.is_empty()
            && self.v_inputs[0].output_index == -1
    }

    // Signing Process:
    // The Transaction, private key of the sender and the prev Transx of the input UTXOs are provided
    // You create a copy of the transaction without any signature or sender's address in the TXInput
    // Iterate over each input UTXO and clear the signature and puts the sender's address in the TXInput
    // Hash the tx_copy and sets the hash as it tx_copy.id
    // Trick:: When the tx_copy got hashed none of the the other input UTXOs had their pub_key filled in except the input which you are signing
    // Clear the input UTXOs pub_key which is being signed
    // Create the signature using tx_copy.id and the sender's private key
    // Fill the signature in the actual Transaction's input UTXO
    pub fn sign(
        &mut self,
        private_key: &[u8],
        prev_txs: HashMap<String, Transaction>,
    ) -> Result<()> {
        // prev_txs is the HashMap of all transactions from where the inputs of this transaction are coming from.

        if self.is_coinbase() {
            return Ok(());
        }

        // Checks that all the inputs are valid
        for tx_input in &self.v_inputs {
            if prev_txs.get(&tx_input.txid).unwrap().id.is_empty() {
                return Err(format_err!("ERROR: Previous transaction is not correct"));
            }
        }

        // Creates a copy of the transaction with empty signature in v_inputs
        let mut tx_copy = self.trim_copy();

        for input_index in 0..tx_copy.v_inputs.len() {
            // Get the prev trx which contained this input
            let prev_tx = prev_txs.get(&tx_copy.v_inputs[input_index].txid).unwrap();

            // Clear the signature of each input UTXO
            tx_copy.v_inputs[input_index].signature.clear();

            // Puts the sender's PKH in the TXInput.pub_key
            tx_copy.v_inputs[input_index].pub_key = prev_tx.v_outputs
                [tx_copy.v_inputs[input_index].output_index as usize]
                .pub_key_hash
                .clone();

            // SHA-256 hash the tx_copy{input UTXOs, output UTXOs} and sets it as its id
            tx_copy.id = tx_copy.hash()?;

            // Clears the pub_key of input UTXOs
            tx_copy.v_inputs[input_index].pub_key = Vec::new();

            // Create a signature using the tx_copy.id and the private_key of the sender
            let signature = ed25519::signature(tx_copy.id.as_bytes(), private_key);

            // Fill the signature of the input UTXO of the actual transaction
            self.v_inputs[input_index].signature = signature.to_vec();
        }

        Ok(())
    }

    // Verify that the input UTXOs are correctly signed
    // Mostly same as the sign function
    pub fn verify(&mut self, prev_txs: HashMap<String, Transaction>) -> Result<bool> {
        if self.is_coinbase() {
            return Ok(true);
        }

        for tx_input in &self.v_inputs {
            if prev_txs.get(&tx_input.txid).unwrap().id.is_empty() {
                return Err(format_err!("ERROR: PREVIOUS TRANSACTION IS NOT CORRECT"));
            }
        }

        let mut tx_copy = self.trim_copy();

        for input_index in 0..self.v_inputs.len() {
            let prev_tx = prev_txs.get(&self.v_inputs[input_index].txid).unwrap();
            tx_copy.v_inputs[input_index].signature.clear();
            tx_copy.v_inputs[input_index].pub_key = prev_tx.v_outputs
                [self.v_inputs[input_index].output_index as usize]
                .pub_key_hash
                .clone();
            tx_copy.id = tx_copy.hash()?;
            tx_copy.v_inputs[input_index].pub_key = Vec::new();

            if !ed25519::verify(
                &tx_copy.id.as_bytes(),
                &self.v_inputs[input_index].pub_key,
                &self.v_inputs[input_index].signature,
            ) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn hash(&mut self) -> Result<String> {
        self.id = String::new();
        let data = bincode::serialize(self)?;
        let mut hasher = Sha256::new();
        hasher.input(&data[..]);
        Ok(hasher.result_str())
    }

    // Creates a copy of the transaction with any signature in any of the inputs
    fn trim_copy(&self) -> Transaction {
        let mut v_inputs = Vec::new();
        let mut v_outputs = Vec::new();

        for tx_input in &self.v_inputs {
            v_inputs.push(TXInput {
                txid: tx_input.txid.clone(),
                output_index: tx_input.output_index.clone(),
                signature: Vec::new(),
                pub_key: Vec::new(),
            })
        }
        for tx_output in &self.v_outputs {
            v_outputs.push(TXOutput {
                value: tx_output.value,
                pub_key_hash: tx_output.pub_key_hash.clone(),
            })
        }

        Transaction {
            id: self.id.clone(),
            v_inputs,
            v_outputs,
        }
    }
}
