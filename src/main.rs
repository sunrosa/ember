use entity::Player;

use crate::entity::{Fire, ItemId};

mod entity;
mod math;

fn main() {
    let mut fire = Fire::init();
    fire.add_item(ItemId::Log).unwrap();

    for i in 0..100 {
        println!("{fire:?}");
        fire.tick();
    }
}
