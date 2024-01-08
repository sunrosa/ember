use entity::Player;

use crate::entity::{Fire, ItemId};

mod entity;
mod math;

fn main() {
    let mut fire = Fire::init()
        .add_item(ItemId::SmallStick)
        .unwrap()
        .add_item(ItemId::SmallStick)
        .unwrap()
        .add_item(ItemId::SmallStick)
        .unwrap()
        .add_item(ItemId::SmallStick)
        .unwrap()
        .add_item(ItemId::SmallStick)
        .unwrap();
    fire = fire.set_tick_resolution(5.0);
    for i in 0..75 {
        if i == 1 {
            fire = fire
                .add_item(ItemId::MediumStick)
                .unwrap()
                .add_item(ItemId::MediumStick)
                .unwrap()
        }
        if i == 3 {
            fire = fire
                .add_item(ItemId::MediumStick)
                .unwrap()
                .add_item(ItemId::MediumStick)
                .unwrap()
        }
        if i == 5 {
            fire = fire.add_item(ItemId::MediumStick).unwrap()
        }
        if i == 8 {
            fire = fire.add_item(ItemId::MediumLog).unwrap();
        }

        println!("{}", fire.summary());
        fire = fire
            .tick()
            .tick()
            .tick()
            .tick()
            .tick()
            .tick()
            .tick()
            .tick()
            .tick()
            .tick();
    }
}
