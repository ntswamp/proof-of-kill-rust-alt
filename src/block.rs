use crate::fight::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
enum State {
    UNVERIFIED,
    VERIFIED,
    FIGHTABLE,
    SEALED,
}


#[derive(Serialize, Deserialize)]
pub struct Block {
    state: State,
    earlier_world_shape:String,
    height: u128,
    fight: Vec<FightResult>
}