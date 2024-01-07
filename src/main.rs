use entity::Player;

use crate::entity::{Fire, ItemId};

mod entity;
mod math;

fn main() {
    let mut fire = Fire::init();

    for i in 0..100 {
        println!("{fire:?}");
        fire.tick();
    }
}
