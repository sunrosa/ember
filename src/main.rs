use entity::FireError;
use entity::{Fire, ItemId::*};
use inquire::{validator::Validation, CustomType, Select};

mod entity;
mod math;

fn main() {
    debug_fire()
}

fn debug_fire() {
    println!(
        "Keep your fire alive. Fire information will be updated each turn. Add \"None\" to \
         progress the turn. If you add too much to your fire at once, it will steal its thermal \
         energy and it will go out. If you don't add to the fire quickly enough, it will go out \
         to fuel exhaustion.\n\nThis is in no way a completed build of the game. This is just a \
         debugger for the fire mechanics.\n\nSelect \"Quit \
         game\" to quit.\n"
    );

    // The number of ticks between turns
    let ticks_per_turn = 5;

    let mut fire = Fire::init();
    let mut quitting_game = false;
    let mut time_at_quit_game = None;
    let mut skip_heating = false;
    let mut time_at_skipped_heating = None;
    loop {
        // Use below for multi-tick approximation for deltas
        // println!("{}", fire.summary_multiple_ticks(ticks_per_turn));
        if !quitting_game {
            println!("{}", fire.summary());
        }

        // Halt the skipping of heating if heating is complete.
        if skip_heating && !fire.has_fresh_items() {
            println!(
                "Skipped {} time",
                fire.time_alive() - time_at_skipped_heating.unwrap()
            );
            time_at_skipped_heating = None;
            skip_heating = false;
        }

        // Bypass user input if they decide to quit or if they decide to skip heating.
        if !(quitting_game || (skip_heating && fire.has_fresh_items())) {
            let selection = Select::new(
                "Add to fire >",
                vec![
                    "None",
                    "Skip heating",
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
                "Quit game" => {
                    quitting_game = true;
                    time_at_quit_game = Some(fire.time_alive());
                    None
                }
                "None" => None,
                "Skip heating" => {
                    skip_heating = true;
                    time_at_skipped_heating = Some(fire.time_alive());
                    None
                }
                "Twig" => Some(Twig),
                "Small stick" => Some(SmallStick),
                "Medium stick" => Some(MediumStick),
                "Large stick" => Some(LargeStick),
                "Medium log" => Some(MediumLog),
                "Large log" => Some(LargeLog),
                e => unreachable!(
                    "Sunrosa made a typo in the prompt match expression. Please report this \
                     incident with ahead context: \"{}\"",
                    e
                ),
            } {
                let count = CustomType::<u32>::new("Add how many >")
                    .with_validator(|x: &u32| {
                        if *x > 200 {
                            Ok(Validation::Invalid(
                                "No more than 200 items at a time.".into(),
                            ))
                        } else {
                            Ok(Validation::Valid)
                        }
                    })
                    .prompt()
                    .unwrap();

                fire = fire.add_items(item, count).unwrap_or_else(|_| {
                    panic!(
                        "Sunrosa fucked up with her fuel definitions. Please report this incident \
                     with the ahead context: \"{:?}\"",
                        item
                    )
                });
            }
        }

        if let Err(FireError::TickAfterDead) = fire.tick_multiple(ticks_per_turn as u32) {
            break;
        }
    }

    println!("{}", fire.summary());
    match time_at_quit_game {
        Some(t) => {
            println!(
                    "Your fire has burned out after {} time! It continued to last {} time after you quit.",
                    fire.time_alive(),
                    fire.time_alive() - t
                );
        }
        None => {
            println!("Your fire has burned out after {} time!", fire.time_alive());
        }
    }
}
