use crate::character::Character;

pub fn versus(fighter1: &mut impl Character,fighter2:&mut impl Character) -> FightResult{

    while fighter1.current_health() > 0 && fighter2.current_health() > 0 {
        //decide the first-mover
        if fighter1.current_action() > fighter2.current_action() {
            fighter2.take_damage(fighter1.produce_damage());
            fighter1.take_damage(fighter2.produce_damage());
        } else {
            fighter1.take_damage(fighter2.produce_damage());
            fighter2.take_damage(fighter1.produce_damage());
        }
        fighter1.regenerate();
        fighter2.regenerate();
        
        fighter1.report_point();
        fighter2.report_point();
    }
    FightResult::make()
    
}


pub struct FightResult {
    winner: String,
}

impl FightResult {
    pub fn make() -> FightResult {
        FightResult {
            winner: String::from("ntswamp"),
        }
    }
}

fn VerifyFight() {

}