use entity::Player;

use crate::entity::{Fire, ItemId};

mod entity;
mod math;

fn main() {
    let mut fire = Fire::init();
    fire.add_item(ItemId::MediumLog).unwrap();
    fire.set_tick_resolution(1.0);

    for i in 0..10000 {
        if i == 5000 {
            fire.add_item(ItemId::MediumLog).unwrap();
        }

        if i % 25 == 0 {
            println!("{fire:?}");
        }

        fire.tick();
    }
}
