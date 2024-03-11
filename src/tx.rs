use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXInput {
    pub txid: String, // Transaction ID of the prev transaction from where the input came from.
    pub output_index: i32, // Index of the output in the previous transaction
    pub script_sig: String, // Signature Script
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXOutput {
    pub value: i32,             // The amount of crypto the that the output holds
    pub script_pub_key: String, // PubKey Script
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
