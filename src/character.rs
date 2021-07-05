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
pub struct Warrior {
    name: String,
    quality: Quality,
    point: Point,
}

#[derive(Debug)]
pub struct Mage {
    name: String,
    quality: Quality,
    point: Point,
}


pub trait Character {
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