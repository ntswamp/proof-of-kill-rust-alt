use crate::character::Character;

pub struct Characterchain {
    pub character: Vec<Character>,
    //restart the game from zero on every lifespan.
    lifespan: usize,
}