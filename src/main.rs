use entity::Player;

use crate::entity::ItemId;

mod entity;

fn main() {
    let mut player = Player::new(100.0);
    player.damage(5.0);

    println!("{player:?}");
}
