const PHYSICAL_DAMAGE_RATE : f32 = 0.5;
const MENTAL_DAMAGE_RATE : f32 = 0.8;

#[derive(Debug)]
struct Point {
    health : i32,
    maxhealth : i32,
    action : i32,
}

#[derive(Debug)]
struct Quality {
    vigor: f32,
    strength: f32,
    agility:f32,
    knowledge:f32,
    toughness:f32,
}

#[derive(Debug)]
struct Warrior {
    name: String,
    quality: Quality,
    point: Point,
}

#[derive(Debug)]
struct Mage {
    name: String,
    quality: Quality,
    point: Point,
}


trait Character {
    fn create(name:String, vigor:f32,strength:f32,agility:f32,knowledge:f32,toughness:f32) -> Self;

    fn report(&self);

    fn current_health(&self) -> i32;

    fn current_action(&self) -> i32;

    fn produce_damage(&self) -> f32;

    fn take_damage(&mut self,damage:f32);

    fn regenerate(&mut self);

    fn die(&self);
}


const WARRIOR_VIGOR_RATE:f32 = 10.0;
const WARRIOR_TOUGHNESS_RATE:f32 = 1.5;
impl Character for Warrior {    
    fn create(name:String, vigor:f32,strength:f32,agility:f32,knowledge:f32,toughness:f32) -> Self {
        Warrior {
            name:name,
            quality: Quality {
                vigor:vigor,
                strength:strength,
                agility:agility,
                knowledge:knowledge,
                toughness:toughness,
            },
            point: Point {
                health: (vigor * WARRIOR_VIGOR_RATE) as i32,
                maxhealth:(vigor * WARRIOR_VIGOR_RATE) as i32,
                action : agility as i32,
            }
        }
    }

    fn report(&self) {
        println!("{:#?}", self);
    }

    fn current_health(&self) -> i32 {
        if self.point.health < 0 {
            self.die();
        }
        self.point.health
    }

    fn current_action(&self) -> i32 {
        self.point.action
    }

    fn produce_damage(&self) -> f32 {
        self.quality.strength * PHYSICAL_DAMAGE_RATE
    }

    fn take_damage(&mut self, damage: f32) {
        self.point.health = self.point.health - damage as i32;
    }


    fn regenerate(&mut self) {
        self.point.health = (self.point.health as f32 + self.quality.toughness * WARRIOR_TOUGHNESS_RATE) as i32;
    }

    fn die(&self) {
        println!("{} is died, game over.", self.name);
    }
}

const MAGE_VIGOR_RATE:f32 = 8.0;
const MAGE_TOUGHNESS_RATE:f32 = 1.0;
impl Character for Mage {    
    fn create(name:String, vigor:f32,strength:f32,agility:f32,knowledge:f32,toughness:f32) -> Self {
        Mage {
            name:name,
            quality: Quality {
                vigor:vigor,
                strength:strength,
                agility:agility,
                knowledge:knowledge,
                toughness:toughness,
            },
            point: Point {
                health: (vigor * MAGE_VIGOR_RATE) as i32,
                maxhealth:(vigor * MAGE_VIGOR_RATE) as i32,
                action : agility as i32,
            }
        }
    }

    fn report(&self) {
        println!("{:#?}", self);
    }

    fn current_health(&self) -> i32 {
        if self.point.health < 0 {
            self.die();
        }
        self.point.health
    }

    fn current_action(&self) -> i32 {
        self.point.action
    }

    fn produce_damage(&self) -> f32 {
        self.quality.knowledge * MENTAL_DAMAGE_RATE
    }

    fn take_damage(&mut self, damage: f32) {
        self.point.health = self.point.health - damage as i32;
    }


    fn regenerate(&mut self) {
        self.point.health = (self.point.health as f32 + self.quality.toughness * MAGE_TOUGHNESS_RATE) as i32;
    }

    fn die(&self) {
        println!("{} is died, game over.", self.name);
    }
}


struct FightResult {
    winner: String,
}

impl FightResult {
    fn make() -> FightResult {
        FightResult {
            winner: String::from("ntswamp"),
        }

    }
}

fn versus(fighter1: &mut impl Character,fighter2:&mut impl Character) -> FightResult{

    while fighter1.current_health() > 0 && fighter2.current_health() > 0 {
        //decide the first-mover
        if fighter1.current_action() > fighter2.current_action() {
            fighter2.take_damage(fighter1.produce_damage());
        } else {
            fighter1.take_damage(fighter2.produce_damage());
        }
        
        fighter1.report();
        fighter2.report();
    }
    FightResult::make()
    
}


fn main() {
    let mut warrior = Warrior::create(String::from("Christopher the Warrior"),100.0,20.0,5.0,1.0,3.0);

    let mut mage = Mage::create(String::from("Collin the Mage"),80.0,1.0,2.0,30.0,1.0);
    
    let result = versus( &mut warrior, &mut mage);
}