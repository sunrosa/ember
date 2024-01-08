use entity::Player;

use crate::entity::{Fire, ItemId};

mod entity;
mod math;

fn main() {
    let mut fire = Fire::init()
        .add_multiple_items(ItemId::SmallStick, 5)
        .unwrap();
    fire = fire.set_tick_resolution(5.0);
    for i in 0..75 {
        if i == 1 {
            fire = fire.add_multiple_items(ItemId::MediumStick, 2).unwrap();
        }
        if i == 3 {
            fire = fire.add_multiple_items(ItemId::MediumStick, 2).unwrap();
        }
        if i == 5 {
            fire = fire.add_item(ItemId::MediumStick).unwrap();
        }
        if i == 8 {
            fire = fire.add_item(ItemId::MediumLog).unwrap();
        }

        println!("{}", fire.summary());
        fire = fire.tick_multiple(10);
    }
}
