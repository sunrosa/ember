use std::collections::HashMap;

use Item::*;

/// The player that plays the game
#[non_exhaustive]
#[derive(Debug, Clone)]
pub(crate) struct Player {
    /// Hit points
    hit_points: f64,
    /// Maximum hit points
    max_hit_points: f64,
    /// Body temperature in degrees kelvin
    body_temperature: f64,
    /// The player's inventory
    inventory: Inventory,
}

impl Player {
    const STARTING_BODY_TEMP: f64 = 310.15;

    pub fn new(max_hp: f64) -> Self {
        Self {
            hit_points: max_hp,
            max_hit_points: max_hp,
            body_temperature: Self::STARTING_BODY_TEMP,
        }
    }

    pub fn damage(&mut self, hp: f64) {
        self.hit_points -= hp;
    }

    pub fn heal(&mut self, hp: f64) {
        self.hit_points += hp;
    }
}

/// An inventory of items
#[non_exhaustive]
#[derive(Debug, Clone)]
pub(crate) struct Inventory {
    /// The type of item held, and the number of that specific item held
    items: HashMap<Item, u32>,
}

/// Static definition for all item types in the game
#[derive(Debug, Clone, Copy)]
pub(crate) enum Item {
    Twig,
    SmallStick,
    MediumStick,
    LargeStick,
    Log,
}

impl Item {
    /// Get an item's mass in grams from static definitions.
    pub fn mass(&self) -> f64 {
        match self {
            Twig => 10.0,
            SmallStick => 500.0,
            MediumStick => 1000.0,
            LargeStick => 2000.0,
            Log => 5000.0,
        }
    }

    /// Get an item's total burn energy for keeping your fire alive.
    pub fn energy(&self) -> f64 {
        match self {
            Twig => 10.0,
            SmallStick => 500.0,
            MediumStick => 1000.0,
            LargeStick => 2000.0,
            Log => 5000.0,
        }
    }

    /// Get an item's chance to hit an enemy when used as a weapon from static definitions.
    pub fn hit_chance(&self) -> f64 {
        match self {
            Twig => 0.10,
            SmallStick => 0.35,
            MediumStick => 0.4,
            LargeStick => 0.5,
            Log => 0.2,
        }
    }

    /// Get an item's hit damage range from static definitions.
    pub fn hit_damage(&self) -> (f64, f64) {
        match self {
            Twig => (0.05, 0.10),
            SmallStick => (2.0, 4.0),
            MediumStick => (4.0, 6.0),
            LargeStick => (8.0, 15.0),
            Log => (8.0, 20.0),
        }
    }
}
