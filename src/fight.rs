use crate::character::Character;
use rand::Rng;

pub fn versus(fighter1: &mut impl Character,fighter2:&mut impl Character) -> FightResult{
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
    FightResult::make(random_seed)
    
}


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

fn VerifyFight(random_seed: Vec<i32>,fighter1:&mut impl Character,fighter2:&mut impl Character) -> FightResult{
    FightResult {
        winner:String::from("unimplemented"),
    }
}