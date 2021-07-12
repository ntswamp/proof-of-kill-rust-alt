use crate::fight::*;

enum State {
    UNVERIFIED,
    VERIFIED,
    FIGHTABLE,
    SEALED,
}



pub struct Block {
    state: State,
    earlier_world_shape:String,
    height: u128,
    fight: Vec<FightResult>
}