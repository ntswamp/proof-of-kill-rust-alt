#[derive(Debug)]
enum Class {
    Warrior,
    Mage,
}

#[derive(Debug)]
struct Character {
    name: String,
    class:Class,
    life: i32,
    strength: i32,
    agility:i32,
}

impl Character {
    fn create(name:String, class:Class,life:i32,strength:i32,agility:i32) -> Character {
        Character {
            name:name,
            class:class,
            life:life,
            strength:strength,
            agility:agility,
        }
    }

    fn report(&self) {
        println!("{:#?}", self);
    }

    fn attack(&self,opponent:&mut Character) {
        opponent.life = opponent.life - self.strength;
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

fn versus(fighter1: &mut Character,fighter2: &mut Character) -> FightResult{

    while fighter1.life > 0 && fighter2.life > 0 {
        //decide the first-mover
        if fighter1.agility > fighter2.agility {
            fighter1.attack(fighter2);
        } else {
            fighter2.attack(fighter1);
        }
        
        fighter1.report();
        fighter2.report();
    }
    FightResult::make()
    
}


fn main() {
    let mut player1 = Character::create(String::from("Christopher"),Class::Warrior,100,20,5);

    let mut player2 = Character::create(String::from("Aming"),Class::Mage,50,40,4);
    
    let result = versus(&mut player1,&mut player2);
}