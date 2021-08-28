/***
 * 
 * TODO:  how do i combine fight and verification into one process?
 * only by this way we can have users a reason to do verification (for winning fights/coins).
 * maybe let the "random seed" which is necessarily used in a fight can be only produced by the verification process?
 * so that before having a fight users must go complete one verification in the first place.
 * (turn the "random seed" into a sort of *ticket* for the fight)
*/
//use rand::Rng;
//use serde::{Serialize, Deserialize};
//use crate::agent::*;

/*

pub fn versus(fighter1: &mut Build,fighter2:&mut Build, random_seed:Option<Vec<i32>>) -> FightResult{
    
    match random_seed {
        None => {
            let mut random_seed : Vec<i32> = vec![];

            let mut rng = rand::thread_rng();
            while fighter1.current_health() > 0 && fighter2.current_health() > 0 {
                //-5 ~ 5 inclusively
                let randomness:i32 = rng.gen_range(-5..=5);
                random_seed.push(randomness);
                //decide the first-mover
                if fighter1.current_action() > fighter2.current_action() {
                    fighter2.take_damage(fighter1.produce_damage(randomness));
                    fighter1.take_damage(fighter2.produce_damage(randomness));
                } else {
                    fighter1.take_damage(fighter2.produce_damage(randomness));
                    fighter2.take_damage(fighter1.produce_damage(randomness));
                }
                fighter1.regenerate();
                fighter2.regenerate();

                fighter1.report_point();
                fighter2.report_point();
            }
            return FightResult::make(random_seed);
        },
        Some(random_seed) => {
            for randomness in &random_seed{
                if fighter1.current_action() > fighter2.current_action() {
                    fighter2.take_damage(fighter1.produce_damage(*randomness));
                    fighter1.take_damage(fighter2.produce_damage(*randomness));
            } else {
                    fighter1.take_damage(fighter2.produce_damage(*randomness));
                    fighter2.take_damage(fighter1.produce_damage(*randomness));
            }
            fighter1.regenerate();
            fighter2.regenerate();

            fighter1.report_point();
            fighter2.report_point();
            }
            return FightResult::make(random_seed);
        },
    }
}

/**
 * 
 * FightResult ought to store a hash which is converted from maybe the entire serialized data of the fight to verify.
 * by doing that, "VerifyFight()" can compare two hashes to check if the local version of the fight's hash differs from the verified one.
 * 
 **/

 #[derive(Serialize, Deserialize)]
pub struct FightResult {
    winner: String,
}

impl FightResult {
    pub fn make(random_seed:Vec<i32>) -> FightResult {
        println!("random sequence:{:#?}",random_seed);
        FightResult {
            winner:String::from("unimplemented"),
        }

    }
}

fn VerifyFight(random_seed: Vec<i32>,fighter1:&mut Build,fighter2:&mut Build, original_result:&FightResult) -> bool{
    //TODO
    true
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_fight() {
        let mut warrior = Warrior::create("Axe the Warrior".to_owned(),100,10,3,2,18);

        let mut mage = Mage::create(String::from("Collin the Mage"),55,2,2,50,5);
    
        let r = vec![
            1,
            -3,
            -2,
            -4,
            -2,
            4,
            4,
            5,
            -5,
            -3,
            2,
            -3,
        ];
        
        let result = versus( &mut warrior, &mut mage, Some(r));
    
    }
}
*/