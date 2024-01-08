use std::collections::HashMap;

use ItemId::*;

use crate::math::weighted_mean;

/// The player that plays the game
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Player {
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
    /// Create a new [`Player`] with customization. See [`init`](Player::init()) to create a [`Player`] with default parameters.
    pub fn new(max_hp: f64, inventory_capacity: f64) -> Self {
        Self {
            hit_points: max_hp,
            max_hit_points: max_hp,
            body_temperature: 310.15,
            inventory: Inventory::new(inventory_capacity),
        }
    }

    /// Create a new _default_ player to start the game with. See the [`new`](Player::new()) function for customization.
    pub fn init() -> Self {
        Self {
            hit_points: 100.0,
            max_hit_points: 100.0,
            body_temperature: 310.15,
            inventory: Inventory::new(10000.0),
        }
    }

    pub fn damage(mut self, hp: f64) -> Self {
        self.hit_points -= hp;
        self
    }

    pub fn heal(mut self, hp: f64) -> Self {
        self.hit_points += hp;
        self
    }
}

/// An inventory of items
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Inventory {
    /// The type of item held, and the number of that specific item held
    items: HashMap<ItemId, u32>,
    /// The inventory's capacity in grams
    capacity: f64,
}

impl Inventory {
    /// Create a new empty inventory.
    ///
    /// # Parameters
    /// * `capacity` - The capacity in grams of the inventory
    pub fn new(capacity: f64) -> Self {
        Inventory {
            items: HashMap::new(),
            capacity,
        }
    }

    /// Get the inventory's capacity in grams
    pub fn capacity(&self) -> f64 {
        self.capacity
    }

    /// Set the inventory's capacity in grams
    pub fn with_capacity(mut self, value: f64) -> Self {
        self.capacity = value;
        self
    }
}

/// Base item data present for every item in the game. Extra, optional, information can be found in more specialized structs such as [`FuelItem`] or [`WeaponItem`]. To store an item properly, combine this struct with whatever specialization you desire, and store it in a tuple or a struct of its own through composition.
///
/// To retrieve item information from asset definitions, use [`ItemId::item()`], [`ItemId::fuel()`], etc.
#[derive(Debug, Clone)]
pub struct Item {
    /// The name of the item, in English, to be served to the player as they play the game.
    pub name: String,
    /// The mass of the item in grams.
    pub mass: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct FuelItem {
    /// The total burn energy of the fuel, in no particular unit. It determines the fuel's burn duration, and also how long it takes to heat up before it burns (in conjunction with [`activation_coefficient`](Self::activation_coefficient)).
    ///
    /// It also affects the fuel's "thermal inertia". If a fresh, cold log is thrown into a fire burning a small stick, it will quickly suck all of the heat from it, because the log has a much higher thermal intertia compared to the stick.
    pub burn_energy: f64,
    /// The fuel's burn temperature in degrees kelvin. The hotter the fuel burns, the faster it'll heat up other fuels for burning. A fire's temperature is the weighted mean of each fuel's [`burn_temperature`](Self::burn_temperature) and each of their [`burn_energy`](Self::burn_energy).
    pub burn_temperature: f64,
    /// The coefficient for the increase in [`activation_progress`](BurningItem::activation_progress) when the fuel is in the heating stage. This does not affect burning in any way.
    pub activation_coefficient: f64,
    /// The minimum temperature for the fuel to gain [`activation_progress`](BurningItem::activation_progress). It will otherwise lose progress. If [`fresh_fuel_radiates`](Fire::fresh_fuel_radiates) is enabled, the fuel will also increase in temperature (and thus absorb less heat from the fire) if the temperature of the fire is above this threshold.
    pub minimum_activation_temperature: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct WeaponItem {
    pub hit_chance: f64,
    pub hit_damage: (f64, f64),
}

/// Here are all item IDs in the game. Contained methods can be used to fetch static item data (like mass and burn temperature). The only thing stored is the item's type. Item data cannot be modified.
#[derive(Debug, Clone, Copy)]
pub enum ItemId {
    Twig,
    SmallStick,
    MediumStick,
    LargeStick,
    MediumLog,
    LargeLog,
    Leaves,
}

impl ItemId {
    /// Get an item's base data from asset definitions.
    pub fn item(&self) -> Item {
        match self {
            Twig => Item {
                name: "twig".into(),
                mass: 10.0,
            },
            SmallStick => Item {
                name: "small stick".into(),
                mass: 500.0,
            },
            MediumStick => Item {
                name: "medium stick".into(),
                mass: 1000.0,
            },
            LargeStick => Item {
                name: "large stick".into(),
                mass: 2000.0,
            },
            MediumLog => Item {
                name: "medium log".into(),
                mass: 3500.0,
            },
            LargeLog => Item {
                name: "large log".into(),
                mass: 5000.0,
            },
            Leaves => Item {
                name: "dry leaf handful".into(),
                mass: 100.0,
            },
        }
    }

    /// Get an item's fuel data from asset definitions. Returns [`None`] if the item is not a [`FuelItem`].
    pub fn fuel(&self) -> Option<FuelItem> {
        match self {
            Twig => Some(FuelItem {
                burn_energy: 10.0,
                burn_temperature: 873.15,
                activation_coefficient: 0.8,
                minimum_activation_temperature: 533.15,
            }),
            SmallStick => Some(FuelItem {
                burn_energy: 500.0,
                burn_temperature: 873.15,
                activation_coefficient: 0.8,
                minimum_activation_temperature: 533.15,
            }),
            MediumStick => Some(FuelItem {
                burn_energy: 1000.0,
                burn_temperature: 873.15,
                activation_coefficient: 0.8,
                minimum_activation_temperature: 533.15,
            }),
            LargeStick => Some(FuelItem {
                burn_energy: 2000.0,
                burn_temperature: 873.15,
                activation_coefficient: 0.8,
                minimum_activation_temperature: 533.15,
            }),
            MediumLog => Some(FuelItem {
                burn_energy: 3500.0,
                burn_temperature: 873.15,
                activation_coefficient: 0.8,
                minimum_activation_temperature: 533.15,
            }),
            LargeLog => Some(FuelItem {
                burn_energy: 5000.0,
                burn_temperature: 873.15,
                activation_coefficient: 0.8,
                minimum_activation_temperature: 533.15,
            }),
            Leaves => Some(FuelItem {
                burn_energy: 100.0,
                burn_temperature: 773.15,
                activation_coefficient: 3.0,
                minimum_activation_temperature: 673.15,
            }),
            _ => None,
        }
    }

    /// Get an item's weapon data from asset definitions. Returns [`None`] if the item is not a [`WeaponItem`].
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

/// An error thrown when trying to construct a [`BurningItem`].
#[derive(Debug, Clone, Copy)]
pub enum BurnItemError {
    /// The item in question is not flammable (or simply lacks needed burn properties in asset definitions).
    NotFlammable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BurnedState {
    Fresh,
    Burning,
    Spent,
}

/// An item that is burning (or is about to be burning) in a fire.
#[derive(Debug, Clone)]
pub struct BurningItem {
    /// The shared item information.
    item: Item,
    /// The item that is burning (or is going to burn in the future)
    fuel: FuelItem,
    /// The amount of energy remaining before the item runs out of energy
    remaining_energy: f64,
    /// The amount of energy put into activating the fuel. When it gets at or above [`Self::remaining_energy`], the fuel will activate. [`Some`] if the fuel has yet to begin burning. [`None`] if the fuel has activated.
    activation_progress: Option<f64>,
    /// Whether the item has activated or not. Once the item beings burning, it will not stop. The item begins burning when [`Self::activation_progress`] reaches its [`Self::remaining_energy`].
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
            item: item_type.item(),
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
            item: item_type.item(),
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
pub struct Fire {
    /// The items that are in the fire's inventory. This includes not-yet-burning items.
    items: Vec<BurningItem>,
    /// The current temperature of the fire. This will not change immediately toward the target temperature, but gradually.
    temperature: f64,
    /// Ambient temperature around the fire
    ambient_temperature: f64,
    /// The amount of time to progress between ticks
    tick_resolution: f64,
    /// Whether items that are [`BurnedState::Fresh`] should get warmer as their activation progress increases. If this is enabled, those items will be able to continue lighting themselves until they start burning without any assistance at all, as long as they're above their [`minimum activation temperature`](FuelItem::minimum_activation_temperature).
    fresh_fuel_radiates: bool,
    /// The amount the fire should include the ambient temperature in its weighted mean of temperature. This simulates heat escaping into the atmosphere.
    weight_of_ambient: f64,
    /// The change in temperature during the last tick.
    temperature_delta: f64,
    /// The change in ambient temperature during the last tick.
    ambient_temperature_delta: f64,
    /// The change in energy remaining during the last tick.
    energy_remaining_delta: f64,
}

/// Getters and setters
impl Fire {
    /// The current temperature of the fire itself
    pub fn temperature(&self) -> f64 {
        self.temperature
    }

    /// The current ambient temperature of the fire itself
    pub fn ambient_temperature(&self) -> f64 {
        self.ambient_temperature
    }

    /// Set the fire's ambient temperature
    pub fn with_ambient_temperature(mut self, value: f64) -> Self {
        self.ambient_temperature = value;
        self
    }

    /// The current tick resolution of the fire
    pub fn tick_resolution(&self) -> f64 {
        self.tick_resolution
    }

    /// Set the amount of time to pass between ticks. Higher resolution means less precision. Don't touch this function unless you know what you're doing.
    pub fn with_tick_resolution(mut self, tick_resolution: f64) -> Self {
        self.tick_resolution = tick_resolution;
        self
    }

    /// Whether items that are [`BurnedState::Fresh`] should get warmer as their activation progress increases. If this is enabled, those items will be able to continue lighting themselves until they start burning without any assistance at all, as long as they're above their [`minimum activation temperature`](FuelItem::minimum_activation_temperature).
    pub fn fresh_fuel_radiates(&self) -> bool {
        self.fresh_fuel_radiates
    }

    /// Whether items that are [`BurnedState::Fresh`] should get warmer as their activation progress increases. If this is enabled, those items will be able to continue lighting themselves until they start burning without any assistance at all, as long as they're above their [`minimum activation temperature`](FuelItem::minimum_activation_temperature).
    pub fn with_fresh_fuel_radiates(mut self, value: bool) -> Self {
        self.fresh_fuel_radiates = value;
        self
    }

    /// The amount the fire should include the ambient temperature in its weighted mean of temperature. This simulates heat escaping into the atmosphere.
    pub fn weight_of_ambient(&self) -> f64 {
        self.weight_of_ambient
    }

    /// The amount the fire should include the ambient temperature in its weighted mean of temperature. This simulates heat escaping into the atmosphere.
    pub fn with_weight_of_ambient(mut self, value: f64) -> Self {
        self.weight_of_ambient = value;
        self
    }

    pub fn ambient_temperature_delta(&self) -> f64 {
        self.ambient_temperature_delta
    }

    pub fn temperature_delta(&self) -> f64 {
        self.temperature_delta
    }

    pub fn energy_remaining_delta(&self) -> f64 {
        self.energy_remaining_delta
    }
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
            temperature: 873.15,
            ambient_temperature: 295.15,
            tick_resolution: 1.0,
            fresh_fuel_radiates: false,
            weight_of_ambient: 3000.0,
            temperature_delta: 0.0,
            energy_remaining_delta: 0.0,
            ambient_temperature_delta: 0.0,
        }
    }

    /// Add a fresh, unburning item to the fire.
    pub fn add_item(mut self, item_type: ItemId) -> Result<Self, BurnItemError> {
        self.items.push(BurningItem::new(item_type)?);

        Ok(self)
    }

    pub fn add_items(mut self, item_type: ItemId, count: u32) -> Result<Self, BurnItemError> {
        for _ in 0..count {
            self = self.add_item(item_type)?;
        }

        Ok(self)
    }

    pub fn summary(&self) -> String {
        let mut output = String::new();

        output += &format!(
            "TEMPERATURE: {:.0}K ({:.2})\nENERGY: {:.0} ({:.2})\n",
            self.temperature(),
            self.temperature_delta(),
            self.energy_remaining(),
            self.energy_remaining_delta(),
        );

        for item in self
            .items
            .iter()
            .filter(|x| x.burned_state == BurnedState::Fresh)
        {
            output += &format!(
                "HEATING {}: {:.0}%\n",
                item.item.name.to_uppercase(),
                item.activation_percentage() * 100.0
            )
        }
        for item in self
            .items
            .iter()
            .filter(|x| x.burned_state == BurnedState::Burning)
        {
            output += &format!(
                "BURNING {}: {:.0}%\n",
                item.item.name.to_uppercase(),
                100.0 * (item.remaining_energy / item.fuel.burn_energy)
            )
        }

        output
    }

    /// The total energy remaining in the fire. This includes both burning and unburning items.
    pub fn energy_remaining(&self) -> f64 {
        let mut output = 0.0;
        for item in &self.items {
            output += item.remaining_energy;
        }

        output
    }

    /// Pass time, and progress all items contained in the fire.
    pub fn tick(mut self) -> Self {
        let ambient_temperature_before = self.ambient_temperature();
        let temperature_before = self.temperature();
        let energy_remaining_before = self.energy_remaining();

        self = self.tick_items();
        self = self.tick_temperature();

        self.ambient_temperature_delta = self.ambient_temperature() - ambient_temperature_before;
        self.temperature_delta = self.temperature() - temperature_before;
        self.energy_remaining_delta = self.energy_remaining() - energy_remaining_before;

        self
    }

    /// Tick `count` times
    pub fn tick_multiple(mut self, count: u32) -> Self {
        for _ in 0..count {
            self = self.tick();
        }

        self
    }

    /// Update the temperature of the entire fire for one tick, depending on [Self::tick_time]. The temperature will jump rapidly toward the target when it's far from the it, but be asymptotic toward it as it gets close. If the number of burning items becomes zero, set the fire's temperature to the ambient temperature. The temperature moves more quickly if the fire has less thermal inertia (energy remaining).
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

        // Add ambient temperature with its configured weight.
        weighted_data.push((self.ambient_temperature(), self.weight_of_ambient()));

        for item in &self.items {
            let temperature = if item.burned_state == BurnedState::Burning {
                item.fuel.burn_temperature
            } else if self.fresh_fuel_radiates()
                && item.burned_state == BurnedState::Fresh
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
        // TODO: Get rid of the clone() call here for efficiency. This may be possible through std's Cell, or clever references.
        for (i, item) in self.items.clone().iter().enumerate() {
            if item.burned_state == BurnedState::Fresh {
                *self.items.get_mut(i).unwrap() = self.heat_item_tick(item);
            } else if item.burned_state == BurnedState::Burning {
                *self.items.get_mut(i).unwrap() = self.burn_item_tick(item);
            }
        }

        // Delete items that have been spent.
        self.items.retain(|x| x.burned_state != BurnedState::Spent);

        self
    }

    /// Tick an unburning item. Items heat up faster if the fire is hotter.
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

    /// Tick a burning item. Items burn faster if the fire is hotter.
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
        let fire = Fire::init().add_items(Twig, 4).unwrap();
        assert_approx_eq!(fire.target_temperature(), 858.137012);
    }

    #[test]
    fn target_temperature_2() {
        let fire = Fire::init().add_item(LargeLog).unwrap();
        assert_approx_eq!(fire.target_temperature(), 428.534615);
    }

    #[test]
    fn fire() {
        let mut fire = Fire::init().add_items(ItemId::SmallStick, 5).unwrap();
        fire = fire.with_tick_resolution(5.0);
        for i in 0..135 {
            if i == 1 {
                fire = fire.add_items(ItemId::MediumStick, 5).unwrap();
            }
            if i == 5 {
                fire = fire.add_items(ItemId::MediumStick, 10).unwrap();
            }
            if i == 5 {
                fire = fire.add_item(ItemId::MediumStick).unwrap();
            }
            if i == 6 {
                fire = fire.add_item(ItemId::MediumLog).unwrap();
            }
            if i == 7 {
                fire = fire.add_item(ItemId::MediumLog).unwrap();
            }
            if i == 8 {
                fire = fire.add_items(ItemId::MediumLog, 2).unwrap();
            }
            if i == 20 {
                fire = fire.add_items(ItemId::MediumLog, 6).unwrap();
            }
            if i == 40 {
                fire = fire.add_items(ItemId::LargeLog, 1).unwrap();
            }

            fire = fire.tick_multiple(15);
        }

        assert_eq!(
            fire.energy_remaining(),
            0.0,
            "The fire should have burned through the entirety of the fuel, but it did not."
        );
    }
}
