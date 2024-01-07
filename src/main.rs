use entity::Player;

use crate::entity::{Fire, ItemId};

mod entity;
mod math;

fn main() {
    let mean = math::weighted_mean(vec![(7.0, 9.0), (5.0, 3.0), (8.0, 2.0), (4.0, 1.0)]);
    println!("{mean}");
}
