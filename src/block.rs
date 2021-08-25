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
use rand::Rng;

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
    agent_id: String,
    agent_build: Build,
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
    pub fn new_block(transactions: Vec<Transaction>, prev_block_hash: String,height: u128) -> Result<Block> {
        let agent = Agent::load().unwrap();
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
            agent_id: "none".to_owned(),
            agent_build: agent.get_build().clone(),
        };
        block.dogfight()?;
        block.agent_id = agent.get_id().to_owned();
        block.agent_build = agent.get_build().to_owned();
        Ok(block)
    }

    /// Run performs a proof-of-work
    fn dogfight(&mut self) -> Result<()> {
        info!("dogfight to the block");
        let transactions = self.transactions.clone();
        for tx in transactions {
            while self.chance != 0  {
                let mut my_build = Agent::load().unwrap().get_build().clone();
                if Block::won(&mut my_build,&mut tx.sender_build.clone(),None) {
                    self.wins += 1;
                }
                self.chance -= 1;
            }
        }
        let data = self.prepare_hash_data()?;
        let mut hasher = Sha256::new();
        hasher.input(&data[..]);
        self.hash = hasher.result_str();
        Ok(())
    }

    /// NewGenesisBlock creates and returns genesis Block
    pub fn new_genesis_block(coinbase: Transaction) -> Block {
        Block::new_block(vec![coinbase], String::new(), 0).unwrap()
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
    fn won(myself: &mut Build, opponent: &mut Build,random_seed:Option<Vec<i32>>) -> bool {
        match random_seed {
            None => {
                let mut random_seed : Vec<i32> = vec![];
    
                let mut rng = rand::thread_rng();
                while myself.current_health() > 0 && opponent.current_health() > 0 {
                    //-5 ~ 5 inclusively
                    let randomness:i32 = rng.gen_range(-5..=5);
                    random_seed.push(randomness);
                    //decide the first-mover
                    if myself.current_action() > opponent.current_action() {
                        opponent.take_damage(myself.produce_damage(randomness));
                        myself.take_damage(opponent.produce_damage(randomness));
                    } else {
                        myself.take_damage(opponent.produce_damage(randomness));
                        opponent.take_damage(myself.produce_damage(randomness));
                    }
                    //self.agent_build.regenerate();
                    //opponent.regenerate();
    
                    myself.report_health();
                    opponent.report_health();
                }
                return myself.current_health() > 0;
            },
            Some(random_seed) => {
                //unimplemented
                for randomness in &random_seed{
                    if myself.current_action() > opponent.current_action() {
                        opponent.take_damage(myself.produce_damage(*randomness));
                        myself.take_damage(opponent.produce_damage(*randomness));
                } else {
                    myself.take_damage(opponent.produce_damage(*randomness));
                        opponent.take_damage(myself.produce_damage(*randomness));
                }
                //myself.regenerate();
                //opponent.regenerate();
    
                myself.report_health();
                opponent.report_health();
                }
                return myself.current_health() > 0;
                
            },
        }

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
