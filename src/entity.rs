use std::collections::HashMap;

use ItemId::*;

use crate::math::weighted_mean;

/// Room temperature in degrees of kelvin
const ROOM_TEMP: f64 = 295.15;

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
    items: HashMap<ItemId, u32>,
}

impl Inventory {
    pub fn new() -> Self {
        Inventory {
            items: HashMap::new(),
        }
    }
}

/// Here are all item IDs in the game. Contained methods can be used to fetch static item data (like mass and burn temperature). The only thing stored is the item's type. Item data cannot be modified.
#[derive(Debug, Clone, Copy)]
pub(crate) enum ItemId {
    Twig,
    SmallStick,
    MediumStick,
    LargeStick,
    Log,
    Leaf,
}

impl ItemId {
    /// Get an item's mass in grams from static definitions.
    pub fn mass(&self) -> f64 {
        match self {
            Twig => 10.0,
            SmallStick => 500.0,
            MediumStick => 1000.0,
            LargeStick => 2000.0,
            Log => 5000.0,
            Leaf => 10.0,
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
            Leaf => Some(10.0),
        }
    }

    /// Get an item's burn temperature in degrees of kelvin from static definitions, if it can burn at all. Returns [None] if the item cannot burn.
    pub fn burn_temperature(&self) -> Option<f64> {
        match self {
            Twig => Some(873.15),
            SmallStick => Some(873.15),
            MediumStick => Some(873.15),
            LargeStick => Some(873.15),
            Log => Some(873.15),
            Leaf => Some(773.15),
        }
    }

    /// Get an item's activation coefficient (to burn) from static definitions, if it can burn at all. Returns [None] if the item cannot burn. This number will use the be multiplied with burn energy to determine the amount of time and temperature to light the item. Gasoline, for example, will have a low activation coefficient. Leaves, on the other hand, have a higher activation coefficient.
    pub fn activation_coefficient(&self) -> Option<f64> {
        match self {
            Twig => Some(1.0),
            SmallStick => Some(1.0),
            MediumStick => Some(1.0),
            LargeStick => Some(1.0),
            Log => Some(1.0),
            Leaf => Some(3.0),
        }
    }

    /// Get an item's chance to hit an enemy when used as a weapon from static definitions. Returns [None] if the item could not be used as a weapon.
    pub fn hit_chance(&self) -> Option<f64> {
        match self {
            Twig => None,
            SmallStick => Some(0.35),
            MediumStick => Some(0.4),
            LargeStick => Some(0.5),
            Log => Some(0.2),
            Leaf => None,
        }
    }

    /// Get an item's hit damage range from static definitions. Returns [None] if the item could not be used as a weapon.
    pub fn hit_damage(&self) -> Option<(f64, f64)> {
        match self {
            Twig => None,
            SmallStick => Some((2.0, 4.0)),
            MediumStick => Some((4.0, 6.0)),
            LargeStick => Some((8.0, 15.0)),
            Log => Some((8.0, 20.0)),
            Leaf => None,
        }
    }
}

/// An error thrown when trying to construct a [BurningItem].
#[derive(Debug, Clone, Copy)]
pub(crate) enum BurnItemError {
    /// The item in question is not flammable (or simply lacks needed burn properties in static definitions).
    NotFlammable,
}

/// An item that is burning (or is about to be burning) in a fire.
#[derive(Debug, Clone)]
pub(crate) struct BurningItem {
    /// The type of the item that is burning
    item_type: ItemId,
    /// The amount of energy remaining before the item runs out of energy
    remaining_energy: f64,
    /// The amount of energy put into activating the fuel. When it gets at or above [Self::remaining_energy], the fuel will activate. [Some] if the fuel has yet to begin burning. [None] if the fuel has activated.
    activation_progress: Option<f64>,
    /// Whether the item has activated or not. Once the item beings burning, it will not stop. The item begins burning when [Self::activation_progress] reaches its [Self::remaining_energy].
    is_burning: bool,
}

impl BurningItem {
    /// Create a new item that has not yet started to burn, and has full remaining percentage.
    pub fn new(item_type: ItemId) -> Result<Self, BurnItemError> {
        let Some(burn_energy) = item_type.burn_energy() else {
            return Err(BurnItemError::NotFlammable);
        };

        Ok(BurningItem {
            item_type,
            remaining_energy: burn_energy,
            activation_progress: Some(0.0),
            is_burning: false,
        })
    }

    /// Create a new item that is already burning, and has a remaining percentage of energy between 0.0 and 1.0. This is used to construct the initial fire when the player begins the game.
    pub fn new_already_burning(
        item_type: ItemId,
        remaining_percentage: f64,
    ) -> Result<Self, BurnItemError> {
        let Some(burn_energy) = item_type.burn_energy() else {
            return Err(BurnItemError::NotFlammable);
        };

        Ok(BurningItem {
            item_type,
            remaining_energy: burn_energy * remaining_percentage,
            activation_progress: None,
            is_burning: true,
        })
    }
}

/// # Design
/// The fire will be maintained solely by fuel the player throws in to keep it alive, continuing to burn while they are asleep. Fuel will be the primary resource for survival in the game. Fuels will have different burn-temperatures (thus burn-speeds) and available energies. Low-temperature, high-energy fuel will have to be thrown in before the player goes to sleep for the night. Fuels will have activation temperatures that will have to be met for a certain duration before they will start burning on their own. For example, kindling like twigs will light almost immediately, while logs will require high temperatures for long durations before they will begin burning themselves. Once a fuel begins burning, it cannot be stopped (at least for this version). The fire will have a list of items, like the player's inventory, and their burn information will be stored and managed there. A fire will be as hot as the total remaining burn energy of items burning with a coefficient to each of their burn temperatures. Items will burn faster if they are in a hotter fire.
///
/// # Ideas
/// * The player will be able to choose their sleep hours. If they choose to sleep at night, they will have to put more fuel into their fire, because nights are colder, however it is easier to find fuel during the day when the sun is up. On the contrary, days are brighter and hotter (and perhaps harder to sleep in), and thus less fuel will be required, but it will be harder to forage at night.
#[derive(Debug, Clone)]
pub(crate) struct Fire {
    /// The items that are in the fire's inventory. This includes not-yet-burning items.
    burning_items: Vec<BurningItem>,
    /// The amount of time to progress between ticks
    tick_time: f64,
    /// The current temperature of the fire. This will not change immediately toward the target temperature, but gradually.
    temperature: f64,
}

impl Fire {
    /// Create a new fire for use at the start of the game. This function should only be called once.
    pub fn init() -> Self {
        Fire {
            burning_items: vec![
                BurningItem::new_already_burning(MediumStick, 0.5).unwrap(),
                BurningItem::new_already_burning(MediumStick, 0.5).unwrap(),
                BurningItem::new_already_burning(MediumStick, 0.5).unwrap(),
            ],
            tick_time: 1.0,
            temperature: 873.15,
        }
    }

    /// Add a fresh, unburning item to the fire.
    pub fn add_item(&mut self, item_type: ItemId) -> Result<(), BurnItemError> {
        self.burning_items.push(BurningItem::new(item_type)?);

        Ok(())
    }

    /// Set the amount of time to pass between ticks. Higher resolution means less precision. Don't touch this function unless you know what you're doing.
    pub fn set_tick_resolution(&mut self, tick_resolution: f64) {
        self.tick_time = tick_resolution;
    }

    /// Pass time, and progress all items contained in the fire.
    pub fn tick() {
        todo!()
    }

    /// Update the temperature of the fire for one tick, depending on [Self::tick_time]. The temperature will jump rapidly toward the target when it's far from the it, but be asymptotic toward it as it gets close.
    fn tick_temperature(&mut self, target_temperature: f64) {
        let temperature_difference = target_temperature - self.temperature;
        self.temperature = self.temperature + ((temperature_difference / 24.0) * self.tick_time);
    }

    /// The temperature the fire would be burning at, dependent on its current items, if it had no thermal mass.
    fn target_temperature(&self) -> f64 {
        let mut weighted_data: Vec<(f64, f64)> = Vec::new();

        for item in &self.burning_items {
            let temperature = if item.is_burning {
                item.item_type.burn_temperature().unwrap()
            } else {
                ROOM_TEMP /* Room temperature plus... */
                    + ((item.item_type.burn_temperature().unwrap() - ROOM_TEMP /* ...the amount above room temperature that the item burns... */)
                        * item.activation_progress.unwrap()) /* ...multiplied by its activation progress */
            };

            weighted_data.push((temperature, item.remaining_energy));
        }

        weighted_mean(weighted_data)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn target_temperature_0() {
        assert_eq!(Fire::init().target_temperature(), 873.15);
    }

    #[test]
    fn target_temperature_1() {
        let mut fire = Fire::init();
        fire.add_item(Twig).unwrap();
        fire.add_item(Twig).unwrap();
        fire.add_item(Twig).unwrap();
        fire.add_item(Twig).unwrap();
        assert_approx_eq!(fire.target_temperature(), 858.137012);
    }

    #[test]
    fn target_temperature_2() {
        let mut fire = Fire::init();
        fire.add_item(Log).unwrap();
        assert_approx_eq!(fire.target_temperature(), 428.534615)
    }
}
