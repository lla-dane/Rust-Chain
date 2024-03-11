use std::collections::HashMap;

use bitcoincash_addr::Address;
use log::debug;
use serde::{Deserialize, Serialize};

use crate::transaction::Transaction;
use crate::errors::Result;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXInput {
    pub txid: String, // Transaction ID of the prev transaction from where the input came from.
    pub output_index: i32, // Index of the output in the previous transaction
    pub signature: Vec<u8>,
    pub pub_key: Vec<u8>, 
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXOutput {
    pub value: i32,             // The amount of crypto the that the output holds
    pub pub_key_hash: Vec<u8>, // PubKey Script
}

impl TXInput {
    // CanUnlockOutputWith checks whether the address initiated the transaction
    pub fn can_unlock_output_with(&self, sender_address: &str) -> bool {
        self.script_sig == sender_address
    }

}

impl TXOutput {

    pub fn new(value: i32, receiver_address: String) -> Result<Self> {
        let mut txo = TXOutput {
            value,
            pub_key_hash: Vec::new(),
        };

        txo.lock(&receiver_address)?;
        Ok(txo)
    }

    fn lock(&mut self, address: &str) -> Result<()> {
        let pub_key_hash = Address::decode(address).unwrap().body;
        debug!("lock: {}", address);
        self.pub_key_hash = pub_key_hash;
        Ok(())
    }

    // CanBeUnlockedWith checks if the output can be unlocked with the provided data
    pub fn can_be_unlocked_with(&self, receiver_address: &[u8]) -> bool {
        self.pub_key_hash == receiver_address
    }
}
