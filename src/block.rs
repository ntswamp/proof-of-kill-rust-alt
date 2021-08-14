use super::*;
use crate::agent::*;
use crate::transaction::Transaction;
use bincode::serialize;
use ::crypto::digest::Digest;
use ::crypto::sha2::Sha256;
use merkle_cbt::merkle_tree::Merge;
use merkle_cbt::merkle_tree::CBMT;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

const TARGET_HEXS: usize = 4;
const CHANCE:u32 = 100;

#[derive(Serialize, Deserialize)]
enum State {
    UNVERIFIED,
    VERIFIED,
    FIGHTABLE,
    SEALED,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block{
    timestamp: u128,
    transactions: Vec<Transaction>,
    hash: String,
    prev_block_hash: String,
    height: u128,
    //chance to have fight with transactions
    chance: u32,
    wins:u32,
    //current champion of this Block
    champion_id: String,
    champion_build: Build,
}

impl Block {
    pub fn get_hash(&self) -> String {
        self.hash.clone()
    }

    pub fn get_wins(&self) -> u32 {
        self.wins
    }

    pub fn get_prev_hash(&self) -> String {
        self.prev_block_hash.clone()
    }

    pub fn get_transaction(&self) -> &Vec<Transaction> {
        &self.transactions
    }

    pub fn get_height(&self) -> u128 {
        self.height
    }

    /// NewBlock creates and returns Block
    pub fn new_block(transactions: Vec<Transaction>, prev_block_hash: String,height: u128,agent:&Agent) -> Result<Block> {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis();
        let mut block = Block {
            timestamp,
            transactions,
            prev_block_hash,
            hash: String::new(),
            height,
            chance:CHANCE,
            wins : 0,
            champion_id: agent.get_id().to_owned(),
            champion_build: agent.get_build().to_owned(),
        };
        //this should be replaced by a function which fill champion data
        block.dogfight()?;
        Ok(block)
    }

    /// Run performs a proof-of-work
    fn dogfight(&mut self) -> Result<()> {
        info!("dogfight to the block");
        for tx in &self.transactions {
            //do tx.sender_build vs own build
        }
        while self.chance != 0  {
            if self.won()? {
                self.wins += 1;
            }
            self.chance -= 1;
        }
        let data = self.prepare_hash_data()?;
        let mut hasher = Sha256::new();
        hasher.input(&data[..]);
        self.hash = hasher.result_str();
        Ok(())
    }

    /// NewGenesisBlock creates and returns genesis Block
    pub fn new_genesis_block(coinbase: Transaction) -> Block {
        let genesis_build: Build = Build::new("Culty".to_owned(),"Warrior".to_owned(),"Warhammer".to_owned());
        let genesis_agent = Agent::new(genesis_build).unwrap();

        Block::new_block(vec![coinbase], String::new(), 0,&genesis_agent).unwrap()
    }

    /// HashTransactions returns a hash of the transactions in the block
    fn hash_transactions(&self) -> Result<Vec<u8>> {
        let mut transactions = Vec::new();
        for tx in &self.transactions {
            transactions.push(tx.hash()?.as_bytes().to_owned());
        }
        let tree = CBMT::<Vec<u8>, MergeVu8>::build_merkle_tree(&transactions);

        Ok(tree.root())
    }

    fn prepare_hash_data(&self) -> Result<Vec<u8>> {
        let content = (
            self.prev_block_hash.clone(),
            self.hash_transactions()?,
            self.timestamp,
            TARGET_HEXS,
            self.chance,
        );
        let bytes = serialize(&content)?;
        Ok(bytes)
    }

    /// won() return true if won the duel.
    fn won(&self) -> Result<bool> {
        let data = self.prepare_hash_data()?;
        let mut hasher = Sha256::new();
        hasher.input(&data[..]);
        let mut vec1: Vec<u8> = Vec::new();
        vec1.resize(TARGET_HEXS, '0' as u8);
        Ok(&hasher.result_str()[0..TARGET_HEXS] == String::from_utf8(vec1)?)
    }
}

struct MergeVu8 {}

impl Merge for MergeVu8 {
    type Item = Vec<u8>;
    fn merge(left: &Self::Item, right: &Self::Item) -> Self::Item {
        let mut hasher = Sha256::new();
        let mut data: Vec<u8> = left.clone();
        data.append(&mut right.clone());
        hasher.input(&data);
        let mut re: [u8; 32] = [0; 32];
        hasher.result(&mut re);
        re.to_vec()
    }
}
