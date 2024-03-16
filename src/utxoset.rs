use log::info;
use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::errors::Result;
use crate::tx::TXOutputs;
use std::collections::HashMap;

// Allows to access the DB connected to our blockchain
pub struct UTXOSet {
    pub blockchain: Blockchain,
}

impl UTXOSet {
    // Reindex rebuils the UTXO set
    pub fn reindex(&self) -> Result<()> {
        if let Err(e) = std::fs::remove_dir_all("data/utxos") {
            info!("not exist any utxos to delete")
        }

        let db = sled::open("data/utxos")?;

        let utxos = self.blockchain.find_utxo();

        for (txid, v_tx_outputs) in utxos {
            db.insert(txid.as_bytes(), bincode::serialize(&v_tx_outputs)?)?;
        }

        Ok(())
    }

    // Updates the UTXO set with transactions from the BLOCk
    // The block is tip of the blockchain
    pub fn update(&self, block: &Block) -> Result<()> {
        let db = sled::open("data/utxos")?;

        for tx in block.get_transactions() {
            if !tx.is_coinbase() {
                for tx_input in &tx.v_inputs {
                    let mut update_outputs = TXOutputs {
                        // This vector will be used to store the remaining unspent outputs
                        // after removing the spent ones
                        outputs: Vec::new(),
                    };
                    let v_tx_outputs: TXOutputs =
                        bincode::deserialize(&db.get(&tx_input.txid)?.unwrap())?;

                    // Think of this line of code from when the first block is added
                    // For the first time the outputs will be put in the v_tx_outputs
                    // in the index sequence only for the first time
                    // BOOYAAH...!!

                    for output_index in 0..v_tx_outputs.outputs.len() {
                        if output_index != tx_input.output_index as usize {
                            update_outputs
                                .outputs
                                .push(v_tx_outputs.outputs[output_index].clone());
                        }
                    }

                    if update_outputs.outputs.is_empty() {
                        db.remove(&tx_input.txid)?;
                    } else {
                        db.insert(
                            tx_input.txid.as_bytes(),
                            bincode::serialize(&update_outputs)?,
                        )?;
                    }
                }
            }

            let mut new_outputs = TXOutputs {
                outputs: Vec::new(),
            };
            for tx_output in &tx.v_outputs {
                new_outputs.outputs.push(tx_output.clone());
            }

            db.insert(tx.id.as_bytes(), bincode::serialize(&new_outputs)?)?;
        }

        Ok(())
    }

    pub fn count_transactions(&self) -> Result<i32> {
        let mut counter = 0;
        let db = sled::open("data/utxos")?;
        for kv in db.iter() {
            kv?;
            counter += 1;
        }
        Ok(counter)
    }

    // Finds the sufficient UTXOs for the transacation to take place
    pub fn find_spendable_outputs(
        &self,
        address: &[u8],
        amount: i32,
    ) -> Result<(i32, HashMap<String, Vec<i32>>)> {
        let mut unspent_outputs: HashMap<String, Vec<i32>> = HashMap::new();
        let mut accumulated = 0;
        let db = sled::open("data/utxos")?;
        for kv in db.iter() {
            let (k, v) = kv?;
            let txid = String::from_utf8(k.to_vec())?;
            let v_tx_outputs: TXOutputs = bincode::deserialize(&v.to_vec())?;

            for output_index in 0..v_tx_outputs.outputs.len() {
                if v_tx_outputs.outputs[output_index].is_locked_with_key(address)
                    && accumulated < amount
                {
                    accumulated += v_tx_outputs.outputs[output_index].value;
                    match unspent_outputs.get_mut(&txid) {
                        Some(v) => v.push(output_index as i32),
                        None => {
                            unspent_outputs.insert(txid.clone(), vec![output_index as i32]);
                        }
                    }
                }
            }
        }
        Ok((accumulated, unspent_outputs))
    }

    // Returns a set of UTXOs for a sender's address
    pub fn find_utxo_for_address(&self, sender_address: &[u8]) -> Result<TXOutputs> {
        let mut utxos = TXOutputs {
            outputs: Vec::new(),
        };
        let db = sled::open("data/utxos")?;

        for kv in db.iter() {
            let (_, v) = kv?;
            let v_tx_outputs: TXOutputs = bincode::deserialize(&v.to_vec())?;

            for utxo in v_tx_outputs.outputs {
                if utxo.can_be_unlocked_with(sender_address) {
                    utxos.outputs.push(utxo.clone())
                }
            }
        }

        Ok(utxos)
    }
}
