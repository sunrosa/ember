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
            inventory: Inventory::new(),
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

impl Inventory {
    pub fn new() -> Self {
        Inventory {
            items: HashMap::new(),
        }
    }
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

    /// Get an item's total burn energy from static definitions (1:1 ratio from grams to energy for natural wood), if it can burn at all. Returns [None] if the item cannot burn.
    pub fn burn_energy(&self) -> Option<f64> {
        match self {
            Twig => Some(10.0),
            SmallStick => Some(500.0),
            MediumStick => Some(1000.0),
            LargeStick => Some(2000.0),
            Log => Some(5000.0),
        }
    }

    /// Get an item's burn temperature from static definitions, if it can burn at all. Returns [None] if the item cannot burn.
    pub fn burn_temperature(&self) -> Option<f64> {
        match self {
            Twig => Some(0.75),
            SmallStick => Some(0.8),
            MediumStick => Some(0.85),
            LargeStick => Some(0.9),
            Log => Some(1.0),
        }
    }

    /// Get an item's activation temperature (to burn) from static definitions, if it can burn at all. Returns [None] if the item cannot burn.
    pub fn activation_temperature(&self) -> Option<f64> {
        match self {
            Twig => Some(0.5),
            SmallStick => Some(0.8),
            MediumStick => Some(1.0),
            LargeStick => Some(1.5),
            Log => Some(4.0),
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

#[derive(Debug, Clone, Copy)]
pub(crate) enum BurnItemError {
    NotFlammable,
}

/// An item that is burning (or is about to be burning) in a fire.
#[derive(Debug, Clone)]
pub(crate) struct BurningItem {
    /// The type of the item that is burning
    item_type: Item,
    /// The amount of energy remaining before the item runs out of energy
    remaining_energy: f64,
}

impl BurningItem {
    pub fn new(item_type: Item) -> Result<Self, BurnItemError> {
        let Some(burn_energy) = item_type.burn_energy() else {
            return Err(BurnItemError::NotFlammable);
        };

        Ok(BurningItem {
            item_type,
            remaining_energy: burn_energy,
        })
    }
}

/// # Design
/// The fire will be maintained solely by fuel the player throws in to keep it alive, including while asleep. Fuel will be the primary resource for survival in the game. Fuels will have different burn-temperatures (thus burn-speeds) and available energies. Low-temperature, high-energy fuel will have to be thrown in before the player goes to sleep for the night. Fuels will have activation temperatures that will have to be met for a certain duration before they will start burning on their own. For example, kindling like twigs will light almost immediately, while logs will require high temperatures for long durations before they will begin burning themselves. Once a fuel begins burning, it cannot be stopped (at least for this version). The fire will have a list of items, like the player's inventory, and their burn information will be stored and managed there. A fire will be as hot as the total remaining burn energy of items burning with a coefficient to each of their burn temperatures. Items will burn faster if they are in a hotter fire.
///
/// # Ideas
/// * The player will be able to choose their sleep hours. If they choose to sleep at night, they will have to put more fuel into their fire, because nights are colder, however it is easier to find fuel during the day when the sun is up. On the contrary, days are brighter and hotter (and perhaps harder to sleep in), and thus less fuel will be required, but it will be harder to forage at night.
#[derive(Debug, Clone)]
pub(crate) struct Fire {
    burning_items: Vec<BurningItem>,
}

impl Fire {
    pub fn new() -> Self {
        Fire {
            burning_items: vec![
                BurningItem::new(MediumStick).unwrap(),
                BurningItem::new(MediumStick).unwrap(),
                BurningItem::new(MediumStick).unwrap(),
            ],
        }
    }
}
