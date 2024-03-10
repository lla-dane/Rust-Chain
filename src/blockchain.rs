// blockchain.rs

use std::collections::HashMap;

use log::info;

use crate::block::{Block, TARGET_HEXT};
use crate::errors::Result;
use crate::transaction::{TXOutput, Transaction};

const GENSIS_COINBASE_DATA: &str =
    "The Times 03/Jan/2009 Chancellor on brink of second bailout for banks";

#[derive(Debug, Clone)]
pub struct Blockchain {
    current_hash: String,
    db: sled::Db,
}

pub struct BlockchainIter<'a> {
    current_hash: String,
    bc: &'a Blockchain,
}

impl Blockchain {

    // Opens the blockchain DB
    pub fn open_blockchain() -> Result<Blockchain> {
        info!("open blockchain");

        let db = sled::open("data/blocks")?;
        let hash = db
            .get("LAST")?
            .expect("Must create a new block database first");
        info!("Found block database");
        let lasthast = String::from_utf8(hash.to_vec())?;

        Ok(Blockchain {
            current_hash: lasthast.clone(),
            db,
        })
    }

    // CreateBlockchain creates a new blockchain DB
    pub fn create_blockchain(address: String) -> Result<Blockchain> {
        info!("Creating new blockchain");

        let db = sled::open("data/blocks")?;
        info!("Creating new block database");
        let cbtx = Transaction::new_coinbase(address, String::from(GENSIS_COINBASE_DATA))?;
        let genesis: Block = Block::new_genesis_block(cbtx);
        db.insert("LAST", genesis.get_hash().as_bytes())?;
        db.insert(genesis.get_hash(), bincode::serialize(&genesis)?)?;
        let bc = Blockchain {
            current_hash: genesis.get_hash(),
            db,
        };
        bc.db.flush()?;
        Ok(bc)
    }

    // Adds block to to the blockchain and blockchain database
    pub fn add_block(&mut self, transactions: Vec<Transaction>) -> Result<()> {
        let lasthash = self.db.get("LAST")?.unwrap();

        let new_block = Block::new_block(transactions, String::from_utf8(lasthash.to_vec())?, TARGET_HEXT)?;
        self.db.insert(new_block.get_hash(), bincode::serialize(&new_block)?)?;
        self.db.insert("LAST", new_block.get_hash().as_bytes())?;
        self.current_hash = new_block.get_hash();
        Ok(())
    }

    // Iterates over the blockchain
    pub fn iter(&self) -> BlockchainIter {
        BlockchainIter {
            current_hash: self.current_hash.clone(),
            bc: &self,
        }
    }

    // Returns a list of all transactions containing UTXOs
    fn find_unspent_transactions(&self, address: &str) -> Vec<Transaction> {

        // String is the txID 
        // Value is the vector of integers containing the index of outputs in the tx that have been spent
        let mut spent_txos: HashMap<String, Vec<i32>> = HashMap::new();

        // Vector to collect transactions that contains unspent outputs for a given address
        let mut unspent_txs: Vec<Transaction> = Vec::new();

        for block in self.iter() {
            for tx in block.get_transactions() {

                for index in 0..tx.v_outputs.len() {
                    // Checks which outputs are spent
                    // by checking id the output index is in the spent_txos hash-map
                    if let Some(indices) = spent_txos.get(&tx.id) {
                        if indices.contains(&(index as i32)) {
                            continue;
                        }
                    }

                    // if output index is not in the the spent_txos hash-map
                    // Push the output to the the unspent_txs if it is to the given address
                    if tx.v_outputs[index].can_be_unlocked_with(address) {
                        unspent_txs.push(tx.to_owned())     // Creates owned data from borrowed data
                    }

                }
                if !tx.is_coinbase() {
                    for tx_input in &tx.v_inputs {
                        if tx_input.can_unlock_output_with(address) {
                            match spent_txos.get_mut(&tx_input.txid) {
                                Some(v) => {
                                    v.push(tx_input.output_index);
                                }
                                None => {
                                    spent_txos.insert(tx_input.txid.clone(), vec![tx_input.output_index]);
                                }
                            }
                        }
                    }
                }               
            }
        }
    unspent_txs
    }

    // Returns a list of all UTXOs 
    pub fn find_utxo(&self, address: &str) -> Vec<TXOutput> {
        let mut utxos = Vec::<TXOutput>::new();
        let unspent_txs = self.find_unspent_transactions(address);
        for tx in unspent_txs {
            for tx_out in &tx.v_outputs {
                if tx_out.can_be_unlocked_with(&address) {
                    utxos.push(tx_out.clone());
                }
            }
        }
        utxos
    }

    // Finds the sufficient UTXOs for the transacation to take place 
    pub fn find_spendable_outputs(&self, address: &str, amount: i32) -> (i32, HashMap<String, Vec<i32>>) {
    let mut unspent_outputs: HashMap<String, Vec<i32>> = HashMap::new();
    let mut accumulated = 0;
    let unspent_txs = self.find_unspent_transactions(address);

    for tx in unspent_txs {
        for index in 0..tx.v_outputs.len() {
            if tx.v_outputs[index].can_be_unlocked_with(address) && accumulated < amount {
                match unspent_outputs.get_mut(&tx.id) {
                    Some(v) => v.push(index as i32),
                    None => {
                        unspent_outputs.insert(tx.id.clone(), vec![index as i32]);
                    }
                }
                accumulated += tx.v_outputs[index].value;

                if accumulated >= amount {
                    return (accumulated, unspent_outputs);
                }
            }
        }
    }
    (accumulated, unspent_outputs)
} 


}




impl<'a> Iterator for BlockchainIter<'a> {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(encode_block) = self.bc.db.get(&self.current_hash) {
            return match encode_block {
                Some(b) => {
                    if let Ok(block) = bincode::deserialize::<Block>(&b) {
                        self.current_hash = block.get_prev_hash();
                        Some(block)
                    } else {
                        None
                    }
                }
                None => None,
            };
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_block() {
        let mut b = Blockchain::create_blockchain(String::new()).unwrap();
        b.add_block(vec![]);
        b.add_block(vec![]);


        for item in b.iter() {
            // Under the hood Rust calls the next method of the Iterator implementation
            // on each iteration.
            println!("item {:?} \n\n", item)
        }
    }
}


