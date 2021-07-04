#[derive(Debug)]
struct Point {
    health : i32,
    health_max : i32,
    action : i32,
    action_max : i32,
    attack : i32,
}

#[derive(Debug)]
struct Quality {
    vigor: i32,
    strength: i32,
    agility:i32,
    knowledge:i32,
    toughness:i32,
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
    fn create(name:String, vigor:i32,strength:i32,agility:i32,knowledge:i32,toughness:i32) -> Self;

    fn report_quality(&self);

    fn report_point(&self);

    fn current_health(&self) -> i32;

    fn current_action(&self) -> i32;

    fn produce_damage(&mut self) -> i32;

    fn take_damage(&mut self,damage:i32);

    fn regenerate(&mut self);

    fn die(&self);
}


//classes
impl Character for Warrior {    
    fn create(name:String, vigor:i32,strength:i32,agility:i32,knowledge:i32,toughness:i32) -> Self {
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
                health: vigor,
                health_max:120,
                action : agility,
                action_max:agility + knowledge,
                attack: strength,
            }
        }
    }

    fn report_quality(&self) {
        println!("{}'s quality:\n{:#?}", self.name,self.quality);
    }

    fn report_point(&self) {
        println!("{}'s current state:\n{:#?}", self.name,self.point);
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

    fn produce_damage(&mut self) -> i32 {
        self.point.action = self.point.action - 2;
        if self.point.action < 0 {
            self.point.action = 0;
            return 0;
        }
        self.point.attack
    }

    fn take_damage(&mut self, damage: i32) {
        self.point.health = self.point.health - damage;
    }


    fn regenerate(&mut self) {
        self.point.health = self.point.health + self.quality.toughness;
        self.point.action = self.point.action + self.quality.agility;
        if self.point.health > self.point.health_max {
            self.point.health = self.point.health_max;
        }
        if self.point.action > self.point.action_max {
            self.point.action = self.point.action_max;
        }
    }

    fn die(&self) {
        println!("{} is died, game over.", self.name);
    }
}


impl Character for Mage {    
    fn create(name:String, vigor:i32,strength:i32,agility:i32,knowledge:i32,toughness:i32) -> Self {
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
                health: vigor,
                health_max:80,
                action : knowledge,
                action_max:agility + strength,
                attack: knowledge,
            }
        }
    }

    fn report_quality(&self) {
        println!("{}'s quality:\n{:#?}", self.name,self.quality);
    }

    fn report_point(&self) {
        println!("{}'s current state:\n{:#?}", self.name,self.point);
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

    fn produce_damage(&mut self) -> i32 {
        self.point.action = self.point.action - 3;
        if self.point.action < 0 {
            self.point.action = 0;
            return 0;
        }
        self.point.attack
    }

    fn take_damage(&mut self, damage: i32) {
        self.point.health = self.point.health - damage;
    }


    fn regenerate(&mut self) {
        self.point.health = self.point.health + self.quality.toughness;
        self.point.action = self.point.action + self.quality.agility;
        if self.point.health > self.point.health_max {
            self.point.health = self.point.health_max;
        }
        if self.point.action > self.point.action_max {
            self.point.action = self.point.action_max;
        }
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


fn main() {
    let mut warrior = Warrior::create("Axe the Warrior".to_owned(),100,20,3,2,8);

    let mut mage = Mage::create(String::from("Collin the Mage"),55,2,2,40,5);
    
    let result = versus( &mut warrior, &mut mage);
}