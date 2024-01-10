use ember::math::BoundedFloat;
use entity::{Fire, ItemId};

mod entity;
mod math;

fn main() {
    let mut rl = rustyline::DefaultEditor::new().unwrap();

    let mut fire = Fire::init().add_items(ItemId::SmallStick, 5).unwrap();
    loop {
        println!("{}", fire.summary());
        let command: String = rl.readline(">> ").unwrap();
        match command.to_lowercase().trim() {
            "twig" => fire = fire.add_item(ItemId::Twig).unwrap(),
            "small stick" => fire = fire.add_item(ItemId::SmallStick).unwrap(),
            "medium stick" => fire = fire.add_item(ItemId::MediumStick).unwrap(),
            "large stick" => fire = fire.add_item(ItemId::LargeStick).unwrap(),
            "medium log" => fire = fire.add_item(ItemId::MediumLog).unwrap(),
            "large log" => fire = fire.add_item(ItemId::LargeLog).unwrap(),
            _ => {}
        }
        fire = fire.tick_multiple(20);
    }
}
