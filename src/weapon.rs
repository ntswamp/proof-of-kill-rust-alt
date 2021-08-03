pub trait Weapon {
    fn get_damage(&self) -> i32;
}

pub struct Sword {
    damage:i32
}
impl Weapon for Sword {
    fn get_damage(&self) -> i32 { self.damage }
}


pub struct Dagger {
    damage:i32
}
impl Weapon for Dagger {
    fn get_damage(&self) -> i32 { self.damage }
}


pub struct Hammer {
    damage:i32
}
impl Weapon for Hammer {
    fn get_damage(&self) -> i32 { self.damage }
}
