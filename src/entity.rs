use std::collections::HashMap;

use ItemId::*;

use crate::math::weighted_mean;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BurnedState {
    Fresh,
    Burning,
    Spent,
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
    burned_state: BurnedState,
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
            burned_state: BurnedState::Fresh,
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
            burned_state: BurnedState::Burning,
        })
    }

    pub fn activation_percentage(&self) -> f64 {
        self.activation_progress.unwrap()
            / (self.item_type.burn_energy().unwrap()
                * self.item_type.activation_coefficient().unwrap())
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
    items: Vec<BurningItem>,
    /// The amount of time to progress between ticks
    tick_time: f64,
    /// The current temperature of the fire. This will not change immediately toward the target temperature, but gradually.
    temperature: f64,
    /// Ambient temperature around the fire
    ambient_temperature: f64,
}

impl Fire {
    /// Create a new fire for use at the start of the game. This function should only be called once.
    pub fn init() -> Self {
        Fire {
            items: vec![
                BurningItem::new_already_burning(MediumStick, 0.5).unwrap(),
                BurningItem::new_already_burning(MediumStick, 0.5).unwrap(),
                BurningItem::new_already_burning(MediumStick, 0.5).unwrap(),
            ],
            tick_time: 1.0,
            temperature: 873.15,
            ambient_temperature: 295.15,
        }
    }

    /// Add a fresh, unburning item to the fire.
    pub fn add_item(&mut self, item_type: ItemId) -> Result<(), BurnItemError> {
        self.items.push(BurningItem::new(item_type)?);

        Ok(())
    }

    /// Set the amount of time to pass between ticks. Higher resolution means less precision. Don't touch this function unless you know what you're doing.
    pub fn set_tick_resolution(&mut self, tick_resolution: f64) {
        self.tick_time = tick_resolution;
    }

    /// The total energy remaining in the fire. This includes both burning and unburning items.
    pub fn energy_remaining(&self) -> f64 {
        let mut output = 0.0;
        for item in &self.items {
            output += item.remaining_energy;
        }

        output
    }

    /// The current temperature of the fire itself
    pub fn temperature(&self) -> f64 {
        self.temperature
    }

    /// Pass time, and progress all items contained in the fire.
    pub fn tick(&mut self) {
        self.tick_items();
        self.tick_temperature();
    }

    /// Update the temperature of the entire fire for one tick, depending on [Self::tick_time]. The temperature will jump rapidly toward the target when it's far from the it, but be asymptotic toward it as it gets close. If the number of burning items becomes zero, set the fire's temperature to the ambient temperature. The temperature moves more quickly if the fire has less thermal inertia (energy remaining).
    fn tick_temperature(&mut self) {
        if self.items.len() != 0 {
            let target_temperature = self.target_temperature();
            let temperature_difference = target_temperature - self.temperature;
            self.temperature = self.temperature
                + ((temperature_difference / (0.024 * self.energy_remaining())) * self.tick_time);
        } else {
            self.temperature = self.ambient_temperature;
        }
    }

    /// The temperature the entire fire would be burning at, dependent on its current items, if it had no thermal intertia. This is the target that the fire will trend toward in its inertia calculation in [Self::tick_temperature()].
    fn target_temperature(&self) -> f64 {
        let mut weighted_data: Vec<(f64, f64)> = Vec::new();

        for item in &self.items {
            let temperature = if item.burned_state == BurnedState::Burning {
                item.item_type.burn_temperature().unwrap()
            } else if item.burned_state == BurnedState::Fresh {
                self.ambient_temperature /* Ambient temperature plus... */
                    + ((item.item_type.burn_temperature().unwrap() - self.ambient_temperature /* ...the amount above room temperature that the item burns... */)
                        * item.activation_percentage()) /* ...multiplied by its activation progress */
            } else {
                unreachable!("The item should not have reached this function in a Spent state.");
            };

            weighted_data.push((temperature, item.remaining_energy));
        }

        weighted_mean(weighted_data)
    }

    /// Tick each item in the fire.
    fn tick_items(&mut self) {
        // The current temperature of the fire.
        let fire_temperature = self.temperature();
        // The current tick time (resolution) of the fire.
        let tick_time = self.tick_time;

        // Modify items.
        for (i, item) in self.items.iter_mut().enumerate() {
            if item.burned_state == BurnedState::Fresh {
                Self::heat_item_tick(item, fire_temperature, tick_time);
            } else if item.burned_state == BurnedState::Burning {
                Self::burn_item_tick(item, fire_temperature, tick_time)
            }
        }

        // Delete items that have been spent.
        self.items.retain(|x| x.burned_state != BurnedState::Spent);
    }

    /// Tick an unburning item.
    fn heat_item_tick(item: &mut BurningItem, fire_temperature: f64, tick_time: f64) {
        *item.activation_progress.as_mut().unwrap() += fire_temperature * 0.001 * tick_time;

        // If the item's activation progress has transcended its activation threshold (burn energy * activation coefficient), set the item to burning, and disable its activation progress.
        if item.activation_progress.unwrap()
            >= item.item_type.burn_energy().unwrap()
                * item.item_type.activation_coefficient().unwrap()
        {
            item.activation_progress = None;
            item.burned_state = BurnedState::Burning;
        }
    }

    /// Tick a burning item. Returns true if the item has extinguished and is spent.
    fn burn_item_tick(item: &mut BurningItem, fire_temperature: f64, tick_time: f64) {
        item.remaining_energy -= fire_temperature * 0.001 * tick_time;

        if item.remaining_energy <= 0.0 {
            item.burned_state = BurnedState::Spent;
            item.remaining_energy = 0.0;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn tick_temperature() {
        let mut fire = Fire::init();
        fire.add_item(Log).unwrap();
        for i in 0..20 {
            match i {
                0 => assert_approx_eq!(fire.temperature(), 873.15),
                4 => assert_approx_eq!(fire.temperature(), 861.858756),
                9 => assert_approx_eq!(fire.temperature(), 848.147083),
                14 => assert_approx_eq!(fire.temperature(), 834.869289),
                19 => assert_approx_eq!(fire.temperature(), 822.011643),
                _ => {}
            }
            fire.tick_temperature();
        }
    }

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
