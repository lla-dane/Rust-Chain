use bitcoincash_addr::Address;
use log::debug;
use serde::{Deserialize, Serialize};

use crate::errors::Result;
use crate::wallet::hash_pub_key;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXInput {
    pub txid: String, // Transaction ID of the prev transaction from where the input came from.
    pub output_index: i32, // Index of the output in the previous transaction
    pub signature: Vec<u8>,
    pub pub_key: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXOutput {
    pub value: i32,            // The amount of crypto the that the output holds
    pub pub_key_hash: Vec<u8>, // Receiver address PKH
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXOutputs {
    pub outputs: Vec<TXOutput>,
}

impl TXInput {
    // CanUnlockOutputWith checks whether the address initiated the transaction
    pub fn can_unlock_output_with(&self, sender_address: &[u8]) -> bool {
        let mut pubkeyhash = self.pub_key.clone();
        hash_pub_key(&mut pubkeyhash);
        pubkeyhash == sender_address
    }
}

impl TXOutput {

    pub fn is_locked_with_key(&self, pub_key_hash: &[u8]) -> bool {
        self.pub_key_hash == pub_key_hash
    }


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
