use lib::character::*;
use lib::fight::*;


fn main() {
    let mut warrior = Warrior::create("Axe the Warrior".to_owned(),100,20,3,2,8);

    let mut mage = Mage::create(String::from("Collin the Mage"),55,2,2,40,5);
    
    let result = versus( &mut warrior, &mut mage);
}