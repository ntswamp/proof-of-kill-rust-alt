use crate::transaction::*;
use crate::agent::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
enum State {
    UNVERIFIED,
    VERIFIED,
    FIGHTABLE,
    SEALED,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    timestamp: u128,
    transactions: Vec<Transaction>,
    prev_block_hash: String,
    hash: String,
    height: i32,
    //rest chance for take up a fight
    chance: i32,
    //current champion of this Block
    champion:Option<String>,
    
}