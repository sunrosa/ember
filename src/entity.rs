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
    SmallStick,
    MediumStick,
    LargeStick,
    Log,
}

impl Into<ItemData> for Item {
    /// Calls Item::new() in order to fetch the item from static item definitions
    fn into(self) -> ItemData {
        ItemData::new(self)
    }
}

/// A physical in-game item's data. Use this type to grab the actual data of the item. If you're simply trying to store the item in memory, use [Item]. For now, this is purely internal. Accesses to an item's data can be done through [Item].
#[non_exhaustive]
#[derive(Debug, Clone)]
struct ItemData {
    /// The item's type
    item_type: Item,
    /// The item's mass in grams
    mass: f64,
}

impl ItemData {
    /// Get a new item from preset item definitions.
    pub fn new(item_type: Item) -> Self {
        match item_type {
            SmallStick => Self {
                item_type: SmallStick,
                mass: 500.0,
            },
            MediumStick => Self {
                item_type: MediumStick,
                mass: 1000.0,
            },
            LargeStick => Self {
                item_type: LargeStick,
                mass: 2000.0,
            },
            Log => Self {
                item_type: Log,
                mass: 5000.0,
            },
        }
    }
}
