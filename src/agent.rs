//! approximately equals to the concept of "wallet" in terms of cryptocurrency

use super::*;
use bincode::{deserialize, serialize};
use bitcoincash_addr::*;
use ::crypto::digest::Digest;
use ::crypto::ed25519;
use ::crypto::ripemd160::Ripemd160;
use ::crypto::sha2::Sha256;
use failure::format_err;
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
    pub name: String,
    pub class: String,
    pub weapon: String,
    
    health: i32,
    attack: i32,
    action: i32,
}
impl Build {
    pub fn new(name: String, class: String, weapon:String) -> Self {
        let health = match &*class {
            "Warrior" => 100,
            "Mage" => 60,
            "Archer" => 80,
            _ => 1,
        };
        let attack = match &*weapon {
            "Axe" => 20,
            "Warhammer" => 15,
            "Wand" => 28,
            "Sword" => 25,
            "Longbow" => 25,
            "Crossbow" => 26,
            _ => 1,
        };
        let action = match &*weapon {
            "Axe" => 20,
            "Warhammer" => 25,
            "Wand" => 35,
            "Sword" => 25,
            "Longbow" => 55,
            "Crossbow" => 40,
            _ => 1,
        };
        Build {
            name,
            class,
            weapon,
            health,
            attack,
            action,
        }
    }
    pub fn introduce(&self) {
        println!("introduce {}:\nthe {} with a(n) {}.", self.name,self.class,self.weapon);
    }

    pub fn report_health(&self) {
        match self.health {
            health if health >= 80 => println!("{} is pretty healthy.",self.name),
            health if health < 80 && health >= 60 => println!("{} is slightly injured.",self.name),
            health if health < 60 && health >= 40 => println!("{} is wounded.",self.name),
            health if health < 40 && health >= 20 => println!("{} is badly hurt.",self.name),
            health if health < 20 => println!("{} is nearly died.",self.name),
            _ => println!("{}'s health is uncertain.",self.name),
        }
    }

    pub fn get_health(&self) -> i32 {
        self.health
    }

    pub fn check_death(&self) -> i32 {
        if self.health <= 0 {
            self.die();
        }
        self.health
    }

    pub fn current_action(&self) -> i32 {
        self.action
    }

    pub fn produce_damage(&mut self, randomness: i32) -> i32 {
        self.action = self.action + randomness;
        if self.action < 0 {
            return 0;
        }
        
        let damage = self.attack + (randomness as f32 * 0.8) as i32;
        //println!("{} deals {} damage to opponent agent.", self.name, damage);
        println!("deals {} damage to opponent agent.", damage);
        damage
    }

    pub fn take_damage(&mut self, damage: i32) {
        //println!("{} took {} damage from opponent agent. Ooouch!", self.name, damage);
        println!("took {} damage from opponent agent. Ooouch!", damage);
        self.health = self.health - damage;
    }

/*
    fn regenerate(&mut self) {
        self.point.health = self.point.health + self.quality.toughness;
        self.point.action = self.point.action + self.quality.agility;
        if self.point.health > self.point.health_max {
            self.point.health = self.point.health_max;
        }
        if self.point.action > self.point.action_max {
            self.point.action = self.point.action_max;
        }
    }
*/
    fn die(&self) {
        println!("{} is died, game over.\n", self.name);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Agent {
    //HashMap<address, keypair>
    addresses : HashMap<String, Keypair>,
    agent_id : String,
    build : Build,
}

impl Agent {
    /// CreateAgent creates Agent and fills it from a file if it exists
    pub fn new(build:Build, node_id:&str) -> Result<Agent> {
        let agent_path = "data_".to_owned() + node_id + "/agent";

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
        let agent = Agent {
            addresses : HashMap::<String, Keypair>::new(),
            agent_id : agent_id,
            build : build
        };
        let db = sled::open(agent_path)?;

        let agent_data = serialize(&agent)?;
        db.insert("MYAGENT", agent_data)?;
        drop(db);
        Ok(agent)
    }

    pub fn load() -> Result<Agent> {
        let node_id =  std::env::var("NODE_ID").unwrap();
        let agent_path = "data_".to_owned() + &node_id + "/agent";
        if !Is_agent_exists(&agent_path) {
            return Err(format_err!("No Existing Agent Found. Create One First."));
        }

        let db = sled::open(agent_path)?;
        let agent_data = db.get("MYAGENT")?.unwrap();
        let mut agent: Agent = deserialize(&agent_data.to_vec())?;

        //load addresses
        for item in db.into_iter() {
            let i = item?;
            if i.0.to_vec() == b"MYAGENT" {
                continue;
            }
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

    /// generate an address for agent
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

    /// save agent and addresses to the disk
    pub fn save(&self) -> Result<()> {
        let node_id = std::env::var("NODE_ID").unwrap();
        let agent_path = "data_".to_owned() + &node_id + "/agent";
        let db = sled::open(agent_path)?;

        for (address, keypair) in &self.addresses {
            let data = serialize(keypair)?;
            db.insert(address, data)?;
        }

        db.flush()?;
        drop(db);
        Ok(())
    }
}

///Returns true if db_path points at an existing entity.
pub fn Is_agent_exists(agent_path: &str) -> bool {
    std::path::Path::new(agent_path).exists()
}

#[cfg(test)]
mod test {
    use super::*;


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
        let build:Build = Build::new (
            "Tim".to_owned(),
            "Warrior".to_owned(),
            "Axe".to_owned(),
        );
        let mut agent1 = Agent::new(build.clone(),"test").unwrap();
        let addr1 = agent1.generate_address();
        let keypair1 = agent1.get_keypair_by_address(&addr1).unwrap().clone();
        agent1.save().unwrap();

        let agent2=  Agent::new(build,"test").unwrap();
        let keypair2 = agent2.get_keypair_by_address(&addr1).unwrap();
        assert_eq!(&keypair1, keypair2);
    }

    #[test]
    #[should_panic]
    fn test_agent_not_exist() {
        let build:Build = Build::new (
            "Tim".to_owned(),
            "Warrior".to_owned(),
            "Axe".to_owned(),
        );
        let k3 = Keypair::new();
        let agent2 = Agent::new(build,"test").unwrap();
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
