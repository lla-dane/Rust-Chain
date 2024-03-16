// blockchain.rs

use std::collections::HashMap;

use failure::format_err;
use log::info;

use crate::block::{Block, TARGET_HEXT};
use crate::errors::Result;
use crate::transaction::Transaction;
use crate::tx::{TXOutput, TXOutputs};

const GENSIS_COINBASE_DATA: &str = "SATOSHI NAKAMOTO";

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
        if let Err(e) = std::fs::remove_dir_all("data/blocks") {
            info!("blocks do not exist to delete")
        }

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
    pub fn add_block(&mut self, transactions: Vec<Transaction>) -> Result<Block> {
        let lasthash = self.db.get("LAST")?.unwrap();

        let new_block = Block::new_block(
            transactions,
            String::from_utf8(lasthash.to_vec())?,
            TARGET_HEXT,
        )?;
        self.db
            .insert(new_block.get_hash(), bincode::serialize(&new_block)?)?;
        self.db.insert("LAST", new_block.get_hash().as_bytes())?;
        self.current_hash = new_block.get_hash();
        Ok(new_block)
    } 

    // Iterates over the blockchain
    pub fn iter(&self) -> BlockchainIter {
        BlockchainIter {
            current_hash: self.current_hash.clone(),
            bc: &self,
        }
    }

    // Returns Transaction with a given transaction id from the whole Blockchain
    pub fn find_tranasaction(&self, id: &str) -> Result<Transaction> {
        for block in self.iter() {
            for tx in block.get_transactions() {
                if tx.id == id {
                    return Ok(tx.clone());
                }
            }
        }
        Err(format_err!("Transaction is not found"))
    }

    // Returns the hash map of the all the prev txs which contained the inputs of the current tx
    fn get_prev_txs(&self, tx: &Transaction) -> Result<HashMap<String, Transaction>> {
        let mut prev_txs = HashMap::new();
        for tx_input in &tx.v_inputs {
            let prev_tx = self.find_tranasaction(&tx_input.txid)?;
            prev_txs.insert(prev_tx.id.clone(), prev_tx);
        }

        Ok(prev_txs)
    }

    // Signs all the input UTXOs of the given transaction
    pub fn sign_transaction(&self, tx: &mut Transaction, private_key: &[u8]) -> Result<()> {
        let prev_txs = self.get_prev_txs(tx)?;
        tx.sign(private_key, prev_txs)?;
        Ok(())
    }

    // Verifies that all the input UTXOs are correctly signed
    pub fn verify_transaction(&self, tx: &mut Transaction) -> Result<bool> {
        let prev_txs = self.get_prev_txs(tx)?;
        tx.verify(prev_txs)
    }

    // Returns a list of all transactions containing UTXOs for a particular address
    fn find_unspent_transactions(&self, address: &[u8]) -> Vec<Transaction> {
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
                        unspent_txs.push(tx.to_owned()) // Creates owned data from borrowed data
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
                                    spent_txos
                                        .insert(tx_input.txid.clone(), vec![tx_input.output_index]);
                                }
                            }
                        }
                    }
                }
            }
        }
        unspent_txs
    }

    pub fn find_utxo(&self) -> HashMap<String, TXOutputs> {
        // String is transaction id which contains utxos
        // TXOutputs contains a vector of tx output
        let mut utxos: HashMap<String, TXOutputs> = HashMap::new();

        // Contains tx ids and index of spent outputs
        let mut spend_txos: HashMap<String, Vec<i32>> = HashMap::new();

        for block in self.iter() {
            for tx in block.get_transactions() {
                for index in 0..tx.v_outputs.len() {
                    if let Some(ids) = spend_txos.get(&tx.id) {
                        if ids.contains(&(index as i32)) {
                            continue;
                        }
                    }

                    match utxos.get_mut(&tx.id) {
                        Some(v_tx_outputs) => {
                            v_tx_outputs.outputs.push(tx.v_outputs[index].clone());
                        }
                        None => {
                            utxos.insert(
                                tx.id.clone(),
                                TXOutputs {
                                    outputs: vec![tx.v_outputs[index].clone()],
                                },
                            );
                        }
                    }

                    if !tx.is_coinbase() {
                        for tx_input in &tx.v_inputs {
                            match spend_txos.get_mut(&tx_input.txid) {
                                Some(v_indices) => {
                                    v_indices.push(tx_input.output_index);
                                }
                                None => {
                                    spend_txos.insert(
                                        tx_input.txid.clone(),
                                        vec![tx_input.output_index.clone()],
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        utxos
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
