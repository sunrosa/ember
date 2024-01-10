#![feature(assert_matches)]

use entity::{Fire, ItemId::*};
use inquire::{CustomType, Select};

mod entity;
mod math;

fn main() {
    debug_fire();
}

fn debug_fire() {
    println!(
        "Keep your fire alive. Fire information will be updated each turn. Add \"None\" to \
         progress the turn. If you add too much to your fire at once, it will steal its thermal \
         energy and it will go out. If you don't add to the fire quickly enough, it will go out \
         to fuel exhaustion\n"
    );

    let mut fire = Fire::init();
    loop {
        println!("{}", fire.summary());

        let selection = Select::new(
            "Add to fire >",
            vec![
                "None",
                "Twig",
                "Small stick",
                "Medium stick",
                "Large stick",
                "Medium log",
                "Large log",
                "Quit game",
            ],
        )
        .prompt();

        if let Some(item) = match selection.unwrap() {
            "Quit game" => break,
            "None" => None,
            "Twig" => Some(Twig),
            "Small stick" => Some(SmallStick),
            "Medium stick" => Some(MediumStick),
            "Large stick" => Some(LargeStick),
            "Medium log" => Some(MediumLog),
            "Large log" => Some(LargeLog),
            e => unreachable!(
                "Sunrosa made a typo in the prompt match expression. Please report this incident \
                 with ahead context: \"{}\"",
                e
            ),
        } {
            let count = CustomType::<u32>::new("Add how many >").prompt().unwrap();

            fire = fire.add_items(item, count).expect(&format!(
                "Sunrosa fucked up with her fuel definitions. Please report this incident with \
                 the ahead context: \"{:?}\"",
                item
            ));
        }

        fire = fire.tick_multiple(5);
    }
}
