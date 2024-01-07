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

#[derive(Debug, Clone, Copy)]
pub(crate) struct Item {
    mass: f64,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct FuelItem {
    burn_energy: f64,
    burn_temperature: f64,
    activation_coefficient: f64,
    minimum_activation_temperature: f64,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct WeaponItem {
    hit_chance: f64,
    hit_damage: (f64, f64),
}

/// Here are all item IDs in the game. Contained methods can be used to fetch static item data (like mass and burn temperature). The only thing stored is the item's type. Item data cannot be modified.
#[derive(Debug, Clone, Copy)]
pub(crate) enum ItemId {
    Twig,
    SmallStick,
    MediumStick,
    LargeStick,
    MediumLog,
    LargeLog,
    Leaf,
}

impl ItemId {
    /// Get an item's mass in grams from static definitions.
    pub fn item(&self) -> Item {
        match self {
            Twig => Item { mass: 10.0 },
            SmallStick => Item { mass: 500.0 },
            MediumStick => Item { mass: 1000.0 },
            LargeStick => Item { mass: 2000.0 },
            MediumLog => Item { mass: 3500.0 },
            LargeLog => Item { mass: 5000.0 },
            Leaf => Item { mass: 10.0 },
        }
    }

    pub fn fuel(&self) -> Option<FuelItem> {
        match self {
            Twig => Some(FuelItem {
                burn_energy: 10.0,
                burn_temperature: 873.15,
                activation_coefficient: 1.0,
                minimum_activation_temperature: 533.15,
            }),
            SmallStick => Some(FuelItem {
                burn_energy: 500.0,
                burn_temperature: 873.15,
                activation_coefficient: 1.0,
                minimum_activation_temperature: 533.15,
            }),
            MediumStick => Some(FuelItem {
                burn_energy: 1000.0,
                burn_temperature: 873.15,
                activation_coefficient: 1.0,
                minimum_activation_temperature: 533.15,
            }),
            LargeStick => Some(FuelItem {
                burn_energy: 2000.0,
                burn_temperature: 873.15,
                activation_coefficient: 1.0,
                minimum_activation_temperature: 533.15,
            }),
            MediumLog => Some(FuelItem {
                burn_energy: 3500.0,
                burn_temperature: 873.15,
                activation_coefficient: 1.0,
                minimum_activation_temperature: 533.15,
            }),
            LargeLog => Some(FuelItem {
                burn_energy: 5000.0,
                burn_temperature: 873.15,
                activation_coefficient: 1.0,
                minimum_activation_temperature: 533.15,
            }),
            Leaf => Some(FuelItem {
                burn_energy: 10.0,
                burn_temperature: 773.15,
                activation_coefficient: 3.0,
                minimum_activation_temperature: 673.15,
            }),
            _ => None,
        }
    }

    pub fn weapon(&self) -> Option<WeaponItem> {
        match self {
            SmallStick => Some(WeaponItem {
                hit_chance: 0.35,
                hit_damage: (2.0, 4.0),
            }),
            MediumStick => Some(WeaponItem {
                hit_chance: 0.4,
                hit_damage: (4.0, 6.0),
            }),
            LargeStick => Some(WeaponItem {
                hit_chance: 0.5,
                hit_damage: (8.0, 15.0),
            }),
            MediumLog => Some(WeaponItem {
                hit_chance: 0.3,
                hit_damage: (6.0, 17.5),
            }),
            LargeLog => Some(WeaponItem {
                hit_chance: 0.2,
                hit_damage: (8.0, 20.0),
            }),
            _ => None,
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
    /// The item that is burning (or is going to burn in the future)
    fuel: FuelItem,
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
        let Some(fuel) = item_type.fuel() else {
            return Err(BurnItemError::NotFlammable);
        };
        let burn_energy = fuel.burn_energy;

        Ok(BurningItem {
            fuel,
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
        let Some(fuel) = item_type.fuel() else {
            return Err(BurnItemError::NotFlammable);
        };

        let burn_energy = fuel.burn_energy;

        Ok(BurningItem {
            fuel,
            remaining_energy: burn_energy * remaining_percentage,
            activation_progress: None,
            burned_state: BurnedState::Burning,
        })
    }

    pub fn activation_percentage(&self) -> f64 {
        self.activation_progress.unwrap()
            / (self.fuel.burn_energy * self.fuel.activation_coefficient)
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
    tick_resolution: f64,
    /// The current temperature of the fire. This will not change immediately toward the target temperature, but gradually.
    temperature: f64,
    /// Ambient temperature around the fire
    ambient_temperature: f64,
}

impl Fire {
    /// Create a new fire for use at the start of the game. This function should only be called once.
    #[must_use]
    pub fn init() -> Self {
        Fire {
            items: vec![
                BurningItem::new_already_burning(MediumStick, 0.5).unwrap(),
                BurningItem::new_already_burning(MediumStick, 0.5).unwrap(),
                BurningItem::new_already_burning(MediumStick, 0.5).unwrap(),
            ],
            tick_resolution: 1.0,
            temperature: 873.15,
            ambient_temperature: 295.15,
        }
    }

    /// Add a fresh, unburning item to the fire.
    pub fn add_item(mut self, item_type: ItemId) -> Result<Self, BurnItemError> {
        self.items.push(BurningItem::new(item_type)?);

        Ok(self)
    }

    /// Set the amount of time to pass between ticks. Higher resolution means less precision. Don't touch this function unless you know what you're doing.
    #[must_use]
    pub fn set_tick_resolution(mut self, tick_resolution: f64) -> Self {
        self.tick_resolution = tick_resolution;

        self
    }

    /// The total energy remaining in the fire. This includes both burning and unburning items.
    pub fn energy_remaining(&self) -> f64 {
        let mut output = 0.0;
        for item in &self.items {
            output += item.remaining_energy;
        }

        output
    }

    /// The current tick resolution of the fire
    pub fn tick_resolution(&self) -> f64 {
        self.tick_resolution
    }

    /// The current temperature of the fire itself
    pub fn temperature(&self) -> f64 {
        self.temperature
    }

    /// The current ambient temperature of the fire itself
    pub fn ambient_temperature(&self) -> f64 {
        self.ambient_temperature
    }

    /// Pass time, and progress all items contained in the fire.
    #[must_use]
    pub fn tick(mut self) -> Self {
        self = self.tick_items();
        self = self.tick_temperature();

        self
    }

    /// Update the temperature of the entire fire for one tick, depending on [Self::tick_time]. The temperature will jump rapidly toward the target when it's far from the it, but be asymptotic toward it as it gets close. If the number of burning items becomes zero, set the fire's temperature to the ambient temperature. The temperature moves more quickly if the fire has less thermal inertia (energy remaining).
    #[must_use]
    fn tick_temperature(mut self) -> Self {
        if self.items.len() != 0 {
            let target_temperature = self.target_temperature();
            let temperature_difference = target_temperature - self.temperature;
            self.temperature = self.temperature()
                + ((temperature_difference / (0.024 * self.energy_remaining()))
                    * self.tick_resolution());
        } else {
            self.temperature = self.ambient_temperature();
        }

        self
    }

    /// The temperature the entire fire would be burning at, dependent on its current items, if it had no thermal intertia. This is the target that the fire will trend toward in its inertia calculation in [Self::tick_temperature()].
    fn target_temperature(&self) -> f64 {
        let mut weighted_data: Vec<(f64, f64)> = Vec::new();

        for item in &self.items {
            let temperature = if item.burned_state == BurnedState::Burning {
                item.fuel.burn_temperature
            } else if item.burned_state == BurnedState::Fresh
                && self.temperature() >= item.fuel.minimum_activation_temperature
            {
                self.ambient_temperature() /* Ambient temperature plus... */
                    + ((item.fuel.burn_temperature - self.ambient_temperature() /* ...the amount above room temperature that the item burns... */)
                        * item.activation_percentage()) /* ...multiplied by its activation progress */
            } else {
                self.ambient_temperature()
            };

            weighted_data.push((temperature, item.remaining_energy));
        }

        weighted_mean(weighted_data)
    }

    /// Tick each item in the fire.
    fn tick_items(mut self) -> Self {
        // The current temperature of the fire.
        let fire_temperature = self.temperature();

        // Modify items.
        for (i, item) in self.items.clone().iter().enumerate() {
            if item.burned_state == BurnedState::Fresh {
                self.items.insert(i, self.heat_item_tick(item));
            } else if item.burned_state == BurnedState::Burning {
                self.items.insert(i, self.burn_item_tick(item))
            }
        }

        // Delete items that have been spent.
        self.items.retain(|x| x.burned_state != BurnedState::Spent);

        self
    }

    /// Tick an unburning item.
    fn heat_item_tick(&self, item: &BurningItem) -> BurningItem {
        let mut item = item.clone();

        if self.temperature() >= item.fuel.minimum_activation_temperature {
            // Increase activation progress if the fire temperature is above the minimum activation temperature of the item.
            *item.activation_progress.as_mut().unwrap() +=
                self.temperature() * 0.005 * self.tick_resolution();
        } else {
            // Decay the item's activation progress if the fire temperature is below the minimum activation temperature of the item.
            *item.activation_progress.as_mut().unwrap() -= ((item.fuel.burn_temperature
                - self.ambient_temperature())
                * item.activation_percentage())
                * 0.005
                * self.tick_resolution();
        }

        // If the item's activation progress has transcended its activation threshold (burn energy * activation coefficient), set the item to burning, and disable its activation progress.
        if item.activation_progress.unwrap()
            >= item.fuel.burn_energy * item.fuel.activation_coefficient
            && self.temperature() >= item.fuel.minimum_activation_temperature
        {
            item.activation_progress = None;
            item.burned_state = BurnedState::Burning;
        }

        item
    }

    /// Tick a burning item.
    fn burn_item_tick(&self, item: &BurningItem) -> BurningItem {
        let mut item = item.clone();

        item.remaining_energy -= self.temperature() * 0.001 * self.tick_resolution();

        if item.remaining_energy <= 0.0 {
            item.burned_state = BurnedState::Spent;
            item.remaining_energy = 0.0;
        }

        item
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn tick_temperature() {
        let mut fire = Fire::init().add_item(LargeLog).unwrap();
        for i in 0..20 {
            match i {
                0 => assert_approx_eq!(fire.temperature(), 873.15),
                4 => assert_approx_eq!(fire.temperature(), 861.858756),
                9 => assert_approx_eq!(fire.temperature(), 848.147083),
                14 => assert_approx_eq!(fire.temperature(), 834.869289),
                19 => assert_approx_eq!(fire.temperature(), 822.011643),
                _ => {}
            }
            fire = fire.tick_temperature();
        }
    }

    #[test]
    fn target_temperature_0() {
        assert_eq!(Fire::init().target_temperature(), 873.15);
    }

    #[test]
    fn target_temperature_1() {
        let fire = Fire::init()
            .add_item(Twig)
            .unwrap()
            .add_item(Twig)
            .unwrap()
            .add_item(Twig)
            .unwrap()
            .add_item(Twig)
            .unwrap();
        assert_approx_eq!(fire.target_temperature(), 858.137012);
    }

    #[test]
    fn target_temperature_2() {
        let fire = Fire::init().add_item(LargeLog).unwrap();
        assert_approx_eq!(fire.target_temperature(), 428.534615)
    }
}
