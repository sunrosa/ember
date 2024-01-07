use entity::Player;

use crate::entity::{Fire, ItemId};

mod entity;
mod math;

fn main() {
    let mut fire = Fire::init();
    fire = fire.set_tick_resolution(5.0);
    for i in 0..100 {
        println!("{i}: {:?}", fire.energy_remaining());
        fire = fire.tick();
    }
}
