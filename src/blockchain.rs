use crate::block::*;

pub struct Blockchain {
    block: Vec<Block>,
    shape:String,
    current_height:u128,
}