use std::collections::HashMap;
use bitcoincash_addr::{Address, HashType, Scheme};
use crypto::{digest::Digest, ed25519, ripemd160::Ripemd160, sha2::Sha256};
use log::info;
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::errors::Result;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Wallet {
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
}

pub struct Wallets {
    // Key: Base58 encoding of the public key hash of the wallet in the value
    wallets: HashMap<String, Wallet>
}

impl Wallet {

    // Creates a public and private key pair using eleptical... functions
    fn new() -> Self {

        // The key declaration means it is an array of 32 bytes (u8 integers) all initialised with 0
        let mut key: [u8; 32] = [0; 32];
        OsRng.fill_bytes(&mut key);

        // The private key is 64 bytes and public key is 32 bytes
        let (private_key, public_key) = ed25519::keypair(&key);

        // This vector is just containg the byte sequence of the keys
        // in which each byte of the key is a vector
        // THEREFORE 32 AND 64 VECTORS
        let private_key = private_key.to_vec();
        let public_key = public_key.to_vec();

        Wallet {
            private_key,
            public_key,
        }
    }

    // Returns Base58 encoding of the public key hash
    fn get_address(&self) -> String {
        let mut pub_hash = self.public_key.clone();
        hash_pub_key(&mut pub_hash);
        let address = Address {
            body: pub_hash,
            scheme: Scheme::Base58,
            hash_type: HashType::Script,
            ..Default::default()
        };
        // 0 O 1 I
        // This encoding converts the binary data in address.body into Base58 string
        address.encode().unwrap()
    }

}

// Returns the SHA-256 RIPEMD-160 hash of the public key(public key hash)
pub fn hash_pub_key(pub_key: &mut Vec<u8>) {
    let mut hasher1 = Sha256::new();
    hasher1.input(pub_key);
    hasher1.result(pub_key);
    let mut hasher2 = Ripemd160::new();
    hasher2.input(pub_key);

    // Resize it because SHA -> 32 bytes and RIPEMD -> 20 bytes 
    // Clear out the extra zeroes
    pub_key.resize(20, 0);
    hasher2.result(pub_key);
}

impl Wallets {

    // Gets hash map of all wallets and their Base58 encoding of the public key hash 
    pub fn new() -> Result<Wallets> {
        let mut wlt = Wallets {
            wallets: HashMap::<String, Wallet>::new(),
        };

        let db = sled::open("data/wallets")?;

        for item in db.into_iter() {
            // IVec is wrapper around a vector of bytes(Vec<u8>) 
            // for handing storing and sending binary data
            let i = item?;
            let address = String::from_utf8(i.0.to_vec())?;
            let wallet = bincode::deserialize(&i.1.to_vec())?;
            wlt.wallets.insert(address, wallet);

        }
        drop(db);
        Ok(wlt)
    }

    // Returns the newly Base58 encoding of a PKH of a new wallet and insert them in DB
    pub fn create_wallet(&mut self) -> String {
        let wallet = Wallet::new();
        
        // This address is the Base58 encoding of the public key hash
        let address = wallet.get_address();
        self.wallets.insert(address.clone(), wallet);
        info!("Create wallet: {}", address);
        address
    }

    // Get all the Base58 PKH stored in the DB
    pub fn get_all_address(&self) -> Vec<String> {
        let mut addresses = Vec::new();
        for (address, _) in &self.wallets {
            addresses.push(address.clone());
        }
        addresses
    }

    // Get the wallet(PubK, PrvK) of a given Base58 PKH from the DB
    pub fn get_wallet(&self, address: &str) -> Option<&Wallet> {
        self.wallets.get(address)
    }

    // Saves all the (Base58 PKH, wallets) in wallets in DB
    pub fn save_all(&self) -> Result<()> {
        let db = sled::open("data/wallets")?;

        for (address, wallet) in &self.wallets {
            let data = bincode::serialize(wallet)?;
            db.insert(address, data)?;
        }

        db.flush()?;
        drop(db);
        Ok(())
    }

}