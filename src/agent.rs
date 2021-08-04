//! approximately equals to the concept of "wallet" in terms of cryptocurrency

use super::*;
use bincode::{deserialize, serialize};
use bitcoincash_addr::*;
use ::crypto::digest::Digest;
use ::crypto::ed25519;
use ::crypto::ripemd160::Ripemd160;
use ::crypto::sha2::Sha256;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sled;
use std::collections::HashMap;
use rand::RngCore;

/**
 * 
 * keypairs held by an agent
 * 
 */
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Keypair {
    pub secret_key: Vec<u8>,
    pub public_key: Vec<u8>,
}

impl Keypair {
    /// NewWallet creates and returns a Wallet
    fn new() -> Self {
        let mut key: [u8; 32] = [0; 32];
        let mut rand = rand::rngs::OsRng;
        rand.fill_bytes(&mut key);
        let (secret_key, public_key) = ed25519::keypair(&key);
        let secret_key = secret_key.to_vec();
        let public_key = public_key.to_vec();
        Keypair {
            secret_key,
            public_key,
        }
    }

    /// GetAddress derive address from public key
    pub fn address(&self) -> String {
        let mut pub_hash: Vec<u8> = self.public_key.clone();
        hash_public_key(&mut pub_hash);
        let address = Address {
            body: pub_hash,
            scheme: Scheme::Base58,
            hash_type: HashType::Script,
            ..Default::default()
        };
        address.encode().unwrap()
    }
}

/// HashPubKey hashes public key
pub fn hash_public_key(pub_key: &mut Vec<u8>) {
    let mut hasher1 = Sha256::new();
    hasher1.input(pub_key);
    hasher1.result(pub_key);
    let mut hasher2 = Ripemd160::new();
    hasher2.input(pub_key);
    pub_key.resize(20, 0);
    hasher2.result(pub_key);
}

/**
 * 
 * 
 * Agent
 * 
 * 
 */

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Build {
    name: String,
    health: i32,
    class: String,
    weapon: String,
}

pub struct Agent {
    //HashMap<address, keypair>
    addresses : HashMap<String, Keypair>,
    agent_id : String,
    build : Build,
}

impl Agent {
    /// CreateAgent creates Agent and fills it from a file if it exists
    pub fn new(build:Build) -> Result<Agent> {
        //agent_id is a 256-bit string
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            `1234567890-=\
                            ~!@#$%^&*()_+";
        const ID_LENGTH: usize = 256;
        let mut rng = rand::thread_rng();
        let agent_id: String = (0..ID_LENGTH)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                    CHARSET[idx] as char
                })
            .collect();
        let mut agent = Agent {
            addresses : HashMap::<String, Keypair>::new(),
            agent_id : agent_id,
            build : build
        };
        let db = sled::open("agent")?;

        for item in db.into_iter() {
            let i = item?;
            let address = String::from_utf8(i.0.to_vec())?;
            let keypair = deserialize(&i.1.to_vec())?;
            agent.addresses.insert(address, keypair);
        }
        drop(db);
        Ok(agent)
    }

    pub fn get_id(&self) -> &str {
        &self.agent_id
    }

    pub fn get_build(&self) -> &Build {
        &self.build
    }


    /// CreateWallet adds a Wallet to Wallets
    pub fn generate_address(&mut self) -> String {
        let keypair = Keypair::new();
        let address = keypair.address();
        self.addresses.insert(address.clone(), keypair);
        info!("create address: {}", address);
        address
    }

    /// GetAddresses returns an array of addresses stored in the wallet file
    pub fn get_all_addresses(&self) -> Vec<String> {
        let mut all_addresses = Vec::<String>::new();
        for (address, _) in &self.addresses {
            all_addresses.push(address.clone());
        }
        all_addresses
    }

    /// GetWallet returns a Keypair by its address
    pub fn get_keypair_by_address(&self, address: &str) -> Option<&Keypair> {
        self.addresses.get(address)
    }

    /// SaveToFile saves wallets to a file
    pub fn save(&self) -> Result<()> {
        let db = sled::open("agent")?;

        for (address, keypair) in &self.addresses {
            let data = serialize(keypair)?;
            db.insert(address, data)?;
        }

        db.flush()?;
        drop(db);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    const build:Build = Build {
        name : "Tim".to_owned(),
        health: 100,
        class: "Warrior".to_owned(),
        weapon: "Sword".to_owned(),
    };

    #[test]
    fn test_create_keypair_and_hash() {
        let k1 = Keypair::new();
        let k2 = Keypair::new();
        assert_ne!(k1, k2);
        assert_ne!(k1.address(), k2.address());

        let mut pub2 = k2.public_key.clone();
        hash_public_key(&mut pub2);
        assert_eq!(pub2.len(), 20);
        let pub_key_hash = Address::decode(&k2.address()).unwrap().body;
        assert_eq!(pub_key_hash, pub2);
    }

    #[test]
    fn test_agent() {
        let mut agent1 = Agent::new(build).unwrap();
        let addr1 = agent1.generate_address();
        let keypair1 = agent1.get_keypair_by_address(&addr1).unwrap().clone();
        agent1.save().unwrap();

        let agent2=  Agent::new(build).unwrap();
        let keypair2 = agent2.get_keypair_by_address(&addr1).unwrap();
        assert_eq!(&keypair1, keypair2);
    }

    #[test]
    #[should_panic]
    fn test_agent_not_exist() {
        let k3 = Keypair::new();
        let agent2 = Agent::new(build).unwrap();
        agent2.get_keypair_by_address(&k3.address()).unwrap();
    }

    #[test]
    fn test_signature() {
        let k = Keypair::new();
        let signature = ed25519::signature("test".as_bytes(), &k.secret_key);
        assert!(ed25519::verify(
            "test".as_bytes(),
            &k.public_key,
            &signature
        ));
    }
}