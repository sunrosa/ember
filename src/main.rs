use entity::Player;

use crate::entity::{Fire, ItemId};

mod entity;
mod math;

fn main() {
    let mut fire = Fire::init();
    fire.add_item(ItemId::Log).unwrap();
    fire.set_tick_resolution(10.0);

    for i in 0..1000 {
        if i % 20 == 0 {
            println!("{fire:?}");
        }

        fire.tick();
    }
}
