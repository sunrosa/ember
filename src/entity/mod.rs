use std::collections::HashMap;

use enum_as_inner::EnumAsInner;
use thiserror::Error;
use ItemId::*;

use crate::math::{weighted_mean, BoundedFloat, BoundedFloatError};

use self::asset::AssetError;

mod asset;
mod test;

/// The player that plays the game
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Player {
    /// The player's hit points
    hit_points: BoundedFloat,
    /// Body temperature in degrees kelvin
    body_temperature: f64,
    /// The player's inventory
    inventory: Inventory,
}

impl Player {
    /// Create a new [`Player`] with customization. See [`init`](Player::init()) to create a [`Player`] with default parameters.
    pub fn new(max_hp: f64, inventory_capacity: f64) -> Self {
        Self {
            hit_points: BoundedFloat::new_zero_min(max_hp, max_hp).unwrap(),
            body_temperature: 310.15,
            inventory: Inventory::new(inventory_capacity),
        }
    }

    /// Create a new _default_ player to start the game with. See the [`new`](Player::new()) function for customization.
    pub fn init() -> Self {
        Self {
            hit_points: BoundedFloat::new_zero_min(100.0, 100.0).unwrap(),
            body_temperature: 310.15,
            inventory: Inventory::new(10000.0),
        }
    }

    /// Deal `hp` damage to the player.
    pub fn damage(&mut self, hp: f64) {
        self.hit_points -= hp;
    }

    /// Heal the player for `hp`.
    pub fn heal(&mut self, hp: f64) {
        self.hit_points += hp;
    }

    pub fn inventory_mut(&mut self) -> &mut Inventory {
        &mut self.inventory
    }

    /// Craft an item, if possible, taking the first craftable recipe if there are multiple. This method accounts for all recipes in the global static recipe set, and also for the items in the player's [`inventory`](Self::inventory_mut).
    ///
    /// # Returns
    /// * [`Ok`]\([`InProgressCraft`]) - A recipe has been found and is ready to begin making progress.
    /// * [`Err`]\([`MissingIngredients`](CraftError::MissingIngredients)) - A recipe was found in the global static recipe set, but the player does not have sufficient items with which to craft it.
    /// * [`Err`]\([`NoRecipe`][CraftError::NoRecipe]) - No recipe with the matching product was found.
    pub fn craft(&mut self, item: ItemId) -> Result<InProgressCraft, CraftError> {
        self.craft_with_set(item, asset::recipes())
    }

    /// Implementation of [`Self::craft()`] but with choice for recipe set used. This is unnecessary at the moment, but may be used in the future.
    fn craft_with_set(
        &mut self,
        item: ItemId,
        recipe_set: &'static RecipeSet,
    ) -> Result<InProgressCraft, CraftError> {
        let compatible_recipes = recipe_set.filter_product(item);

        if compatible_recipes.is_empty() {
            return Err(CraftError::NoRecipe(item));
        }

        // Search through each of the recipes found for the specified product, and pick the FIRST that is craftable.
        let mut missing_items = Vec::new();
        for recipe in compatible_recipes {
            match self.inventory.take_vec_if_enough(&recipe.ingredients) {
                Ok(_) => {
                    return Ok(InProgressCraft {
                        products: &recipe.products,
                        time_remaining: recipe.craft_time,
                    });
                }
                Err(InventoryError::NotEnoughVec(e)) => {
                    missing_items = e;
                    continue;
                }
                _ => unreachable!(),
            }
        }

        // No recipes were found that the player can craft.
        Err(CraftError::MissingIngredients(missing_items))
    }
}

/// In order to complete the craft immediately, call [`complete()`](Self::complete()), and it will tick the fire accordingly. If you have limited time to await the craft, call [`progress`](Self::progress()) to progress the craft by a specified amount of time.
///
/// # Development
/// * Allow for canceling of the craft to return the ingredients back to the player (impossible with the current implementation).
#[derive(Clone, Debug)]
pub struct InProgressCraft<'a> {
    products: &'a Vec<(ItemId, u32)>,
    time_remaining: f64,
}

// This really, really reminds me of Futures lol. I forgot what this process is called. "Make invalid states unrepresentable" or some shit. I think it's the Finite-State-Machine pattern. I like it a fucking hell of a lot though :3
impl<'a> InProgressCraft<'a> {
    /// Complete the craft immediately, ticking the fire for however long the craft has remaining, returning the products. This method takes ownership and destroys its receiver.
    pub fn complete(self, fire: &mut Fire) -> Result<&'a Vec<(ItemId, u32)>, FireError> {
        fire.tick_time(self.time_remaining)?;
        Ok(self.products)
    }

    /// Progress the craft by `time` time, "polling" it. This method will take only the time necessary to finish the craft, and not the entire amount of time specified. Because this method takes ownership of its receiver, you will have to use its returned [`CraftResult`] exclusively.
    ///
    /// # Returns
    /// * [`Ready`](CraftResult::Ready) - The craft has completed.
    /// * [`Pending`](CraftResult::Pending) - There is still more time needed to complete the task.
    pub fn progress(mut self, fire: &mut Fire, time: f64) -> Result<CraftResult<'a>, FireError> {
        if time >= self.time_remaining {
            // Ready
            fire.tick_time(self.time_remaining)?;
            return Ok(CraftResult::Ready(self.products));
        } else {
            // Pending
            fire.tick_time(time)?;
            self.time_remaining -= time;
            Ok(CraftResult::Pending(self))
        }
    }
}

/// The result of "polling" a crafting process
#[derive(Debug, Clone, EnumAsInner)]
pub enum CraftResult<'a> {
    /// The craft is ready. Contained are the item products of the recipe.
    Ready(&'a Vec<(ItemId, u32)>),
    /// The craft is still pending. Contained is the in-progress craft to be "polled" again.
    Pending(InProgressCraft<'a>),
}

#[derive(Clone, Debug, Error)]
pub enum CraftError {
    /// The inventory contains insufficient ingredients to craft.
    ///
    /// * `0` - [`Vec`] of Ingredients
    ///     * `0` - Item
    ///     * `1` - Amount
    #[error("Insufficient ingredients to craft: {0:?}.")]
    MissingIngredients(Vec<(ItemId, u32)>),

    /// No compatible recipe was found the specified item.
    ///
    /// * `0` - The item that was attempted to be crafted
    #[error("No compatible recipe found to craft: {0:?}.")]
    NoRecipe(ItemId),
}

#[derive(Clone, Debug, Error)]
pub enum InventoryError {
    /// The item does not exist in the inventory.
    #[error("The item {0:?} does not exist in the inventory.")]
    NotFound(ItemId),

    /// Not enough of the item to be taken from the inventory.
    #[error(
        "Not enough of the item {0:?} to take from the inventory. Count {1} are currently available."
    )]
    NotEnough(ItemId, u32),

    /// Could never store that many of the item even if the inventory were empty.
    #[error("Could never store count {1} of item {0:?}, even when empty.\nTotal capacity: {2}")]
    NoCapacity(ItemId, u32, f64),

    /// The inventory does not have the available capacity to store that many of the item.
    #[error("Not enough available capacity to store count {count} of item {item:?}.\nUsed capacity: {used_capacity}\nTotal capacity: {max_capacity}\n")]
    NoAvailableCapacity {
        item: ItemId,
        count: u32,
        used_capacity: f64,
        max_capacity: f64,
    },

    /// The following [`Vec`] of items are missing.
    #[error("The following items are missing: {0:?}")]
    NotEnoughVec(Vec<(ItemId, u32)>),
}

/// An inventory of items
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Inventory {
    /// The type of item held, and the number of that specific item held
    items: HashMap<ItemId, u32>,
    /// The inventory's used capacity in grams. Bounded to a maximum and a minimum. The minimum is usually `0.0`.
    used_capacity: BoundedFloat,
}

impl Inventory {
    /// Create a new empty inventory.
    ///
    /// # Parameters
    /// * `capacity` - The capacity in grams of the inventory
    pub fn new(capacity: f64) -> Self {
        Inventory {
            items: HashMap::new(),
            used_capacity: BoundedFloat::new(0.0, 0.0, capacity).unwrap(),
        }
    }

    /// Get the inventory's capacity in grams
    pub fn used_capacity(&self) -> BoundedFloat {
        self.used_capacity
    }

    /// Set the inventory's capacity in grams
    pub fn with_max_capacity(mut self, value: f64) -> Result<Self, BoundedFloatError> {
        self.used_capacity = self.used_capacity.with_max(value)?;
        Ok(self)
    }

    /// Insert an item into the inventory.
    ///
    /// # Parameters
    /// * `item` - The item to insert
    /// * `count` - The amount of the item to insert
    pub fn insert(&mut self, item: ItemId, count: u32) -> Result<(), InventoryError> {
        let mass_of_insertion = Item::from(item).mass * count as f64;

        // If the inventory could never store X count of item
        if self.used_capacity().max() < mass_of_insertion {
            return Err(InventoryError::NoCapacity(
                item,
                count,
                self.used_capacity().max(),
            ));
        }

        // If the inventory can't store X count of item with its current available capacity
        if self.used_capacity().max_diff() < mass_of_insertion {
            return Err(InventoryError::NoAvailableCapacity {
                item,
                count,
                used_capacity: self.used_capacity().current(),
                max_capacity: self.used_capacity().max(),
            });
        }

        // Insert the item
        self.used_capacity += mass_of_insertion;
        *self.items.entry(item).or_default() += count;

        Ok(())
    }

    /// Take 1 `item` from the inventory, removing it in-place.
    ///
    /// # Returns
    /// * [`InventoryError::NotEnough`] - if not enough of the item exist in the inventory
    /// * [`InventoryError::NotFound`] - if no record of the item exists in the inventory
    pub fn take_one(&mut self, item: ItemId) -> Result<(), InventoryError> {
        self.take_amount(item, 1)
    }

    /// Take `count` `item`s from the inventory, removing them in-place.
    ///
    /// # Returns
    /// * [`InventoryError::NotEnough`] - if not enough of the item exist in the inventory
    /// * [`InventoryError::NotFound`] - if no record of the item exists in the inventory
    pub fn take_amount(&mut self, item: ItemId, count: u32) -> Result<(), InventoryError> {
        // If none of the item exist in the inventory
        if !self.items.contains_key(&item) {
            return Err(InventoryError::NotFound(item));
        }

        let entry = self.items.entry(item).or_default();

        // If too few items of the chosen kind are in the inventory
        if *entry < count {
            return Err(InventoryError::NotEnough(item, count));
        }

        // Actually subtract the item
        self.used_capacity -= Item::from(item).mass * count as f64;
        *entry = entry.saturating_sub(count);

        // Remove the item from the items hashmap if its count is 0.
        if *entry == 0 {
            self.items.remove(&item);
        }

        Ok(())
    }

    /// Take all of `item` from the inventory. Removing them in-place.
    ///
    /// # Returns
    /// * Ok - The number of items taken
    /// * [`InventoryError::NotFound`] - if a record of the item does not exist in the inventory
    pub fn take_all(&mut self, item: ItemId) -> Result<u32, InventoryError> {
        // If none of the item exist in the inventory
        if !self.items.contains_key(&item) {
            return Err(InventoryError::NotFound(item));
        }

        // Get the amount of items of that certain kind
        let amount = *self.items.get(&item).expect("This should be unreachable.");

        // Remove those items
        self.used_capacity -= Item::from(item).mass * amount as f64;
        self.items.remove(&item);

        Ok(amount)
    }

    /// Does the inventory contain at least `amount` of `item`?
    pub fn contains(&self, item: ItemId, amount: u32) -> bool {
        *self.items.get(&item).unwrap_or(&0) >= amount
    }

    /// Does the inventory contain at least each amount of item in `wanted_items`?
    ///
    /// # Returns
    /// * true - __All__ items are contained in the inventory.
    /// * false - Some of the items are missing.
    pub fn contains_vec(&self, wanted_items: &Vec<(ItemId, u32)>) -> EnoughItems {
        if wanted_items.iter().all(|x| self.contains(x.0, x.1)) {
            EnoughItems::Enough
        } else {
            let mut missing_items = Vec::new();
            for wanted_item in wanted_items {
                let difference = wanted_item.1 - self.items.get(&wanted_item.0).unwrap_or(&0);
                missing_items.push((wanted_item.0, difference));
            }

            EnoughItems::Missing(missing_items)
        }
    }

    /// Take `wanted_items` from this inventory. __Only removes items if all items necessary are present.__
    ///
    /// # Returns
    /// * [`Ok`] - The items were succesfully taken.
    /// * [`NotEnoughVec`](InventoryError::NotEnoughVec) - The inventory does not contain enough items to be taken. __No items have been removed.__
    pub fn take_vec_if_enough(
        &mut self,
        wanted_items: &Vec<(ItemId, u32)>,
    ) -> Result<(), InventoryError> {
        if let EnoughItems::Missing(i) = self.contains_vec(wanted_items) {
            Err(InventoryError::NotEnoughVec(i))
        } else {
            for item in wanted_items {
                // This unwrap should be unreachable because of the top-level if statement.
                self.take_amount(item.0, item.1).unwrap();
            }

            Ok(())
        }
    }
}

/// Result of checking to see if there are enough items in an inventory to craft a recipe
pub enum EnoughItems {
    /// There are enough items.
    Enough,
    /// The following items are missing. If the inventory partially contains an item necessary, they will be subtracted from this difference.
    Missing(Vec<(ItemId, u32)>),
}

/// Base item data present for every item in the game. Extra, optional, information can be found in more specialized structs such as [`FuelItem`] or [`WeaponItem`]. To store an item properly, combine this struct with whatever specialization you desire, and store it in a tuple or a struct of its own through composition.
///
/// To retrieve item information from asset definitions, use [`ItemId::item()`], [`ItemId::fuel()`], etc.
#[derive(Debug, Clone)]
pub struct Item {
    /// The name of the item, in English, to be served to the player
    pub name: String,
    /// Description of the item, in English, to be served to the player when queried
    pub description: String,
    /// The mass of the item in grams
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
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ItemId {
    Twig,
    SmallStick,
    MediumStick,
    LargeStick,
    MediumLog,
    LargeLog,
    Leaves,
    SmallBundle,
    MediumBundle,
}

/// An error thrown when trying to construct a [`BurningItem`].
#[derive(Debug, Clone, Copy, Error)]
pub enum BurnItemError {
    /// The item in question is not flammable (or simply lacks needed burn properties in asset definitions).
    #[error("{0:?} is not a flammable item.")]
    NotFlammable(ItemId),
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
        let fuel = match FuelItem::try_from(item_type) {
            Ok(o) => o,
            Err(AssetError::NotFound(e)) => return Err(BurnItemError::NotFlammable(e)),
        };

        let burn_energy = fuel.burn_energy;

        Ok(BurningItem {
            item: item_type.into(),
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
        let fuel = match FuelItem::try_from(item_type) {
            Ok(o) => o,
            Err(AssetError::NotFound(e)) => return Err(BurnItemError::NotFlammable(e)),
        };

        let burn_energy = fuel.burn_energy;

        Ok(BurningItem {
            item: item_type.into(),
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
    /// The time that the fire has been alive.
    time_alive: f64,
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

    /// The change in ambient temperature during the last tick.
    pub fn ambient_temperature_delta(&self) -> f64 {
        self.ambient_temperature_delta
    }

    /// The change in temperature during the last tick.
    pub fn temperature_delta(&self) -> f64 {
        self.temperature_delta
    }

    /// The change in energy remaining during the last tick.
    pub fn energy_remaining_delta(&self) -> f64 {
        self.energy_remaining_delta
    }

    /// The amount of time that the fire has spent alive.
    pub fn time_alive(&self) -> f64 {
        self.time_alive
    }
}

impl Fire {
    /// Create a new fire for use at the start of the game. This function should only be called once.
    pub fn init() -> Self {
        Fire {
            items: vec![
                BurningItem::new_already_burning(MediumStick, 0.8).unwrap(),
                BurningItem::new_already_burning(MediumStick, 0.8).unwrap(),
                BurningItem::new_already_burning(MediumStick, 0.8).unwrap(),
            ],
            temperature: 873.15,
            ambient_temperature: 295.15,
            tick_resolution: 1.0,
            fresh_fuel_radiates: false,
            weight_of_ambient: 3000.0,
            temperature_delta: 0.0,
            energy_remaining_delta: 0.0,
            ambient_temperature_delta: 0.0,
            time_alive: 0.0,
        }
    }

    /// Add a fresh, unburning item to the fire.
    ///
    /// # Errors
    /// Returns [`NotFlammable`](BurnItemError::NotFlammable) if the [`ItemId`] passed in is not of a flammable item.
    pub fn add_item(mut self, item_type: ItemId) -> Result<Self, BurnItemError> {
        self.items.push(BurningItem::new(item_type)?);

        Ok(self)
    }

    /// Add [`count`] of the same item to the fire.
    ///
    /// # Errors
    /// Returns [`NotFlammable`](BurnItemError::NotFlammable) if the [`ItemId`] passed in is not of a flammable item.
    pub fn add_items(mut self, item_type: ItemId, count: u32) -> Result<Self, BurnItemError> {
        for _ in 0..count {
            self = self.add_item(item_type)?;
        }

        Ok(self)
    }

    /// Basic summary string for printing out one tick's infomation to a user interface.
    pub fn summary(&self) -> String {
        self.summary_multiple_ticks(1)
    }

    /// Print out a summary with deltas from `ticks` ticks.
    pub fn summary_multiple_ticks(&self, ticks: u32) -> String {
        let mut output = String::new();

        output += &format!(
            "TEMPERATURE: {:.0}K ({:.2})\nBURNING ENERGY: {:.0} ({:.0}%) ({:.2})\nFRESH ENERGY: \
             {:.0} ({:.0}%)\n",
            self.temperature(),
            self.temperature_delta() * ticks as f64,
            self.burning_energy_remaining(),
            self.burning_energy_remaining() / self.energy_remaining() * 100.0,
            self.energy_remaining_delta() * ticks as f64,
            self.fresh_energy_remaining(),
            self.fresh_energy_remaining() / self.energy_remaining() * 100.0,
        );

        output += "===========================\n";

        for (i, item) in self
            .items
            .iter()
            .filter(|x| x.burned_state == BurnedState::Fresh)
            .enumerate()
        {
            if i > 15 {
                output += "...\n";
                break;
            }

            output += &format!(
                "HEATING {}: {:.0}%\n",
                item.item.name.to_uppercase(),
                item.activation_percentage() * 100.0
            )
        }

        output += "===========================\n";

        for (i, item) in self
            .items
            .iter()
            .filter(|x| x.burned_state == BurnedState::Burning)
            .enumerate()
        {
            if i > 15 {
                output += "...\n";
                break;
            }

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

    /// The total energy remaining in _exclusively_ the burning items in the fire.
    pub fn burning_energy_remaining(&self) -> f64 {
        let mut output = 0.0;
        for item in self
            .items
            .iter()
            .filter(|x| x.burned_state == BurnedState::Burning)
        {
            output += item.remaining_energy;
        }

        output
    }

    /// The total energy remaining in _exclusively_ the fresh items in the fire.
    pub fn fresh_energy_remaining(&self) -> f64 {
        let mut output = 0.0;
        for item in self
            .items
            .iter()
            .filter(|x| x.burned_state == BurnedState::Fresh)
        {
            output += item.remaining_energy;
        }

        output
    }

    /// Pass time, and progress all items contained in the fire.
    ///
    /// # Returns
    /// [`TickAfterDead`](FireError::TickAfterDead) - The fire was attempted to be ticked after it had died.
    pub fn tick(&mut self) -> Result<(), FireError> {
        if !self.is_alive() {
            return Err(FireError::TickAfterDead);
        }

        let ambient_temperature_before = self.ambient_temperature();
        let temperature_before = self.temperature();
        let energy_remaining_before = self.energy_remaining();

        self.tick_items();
        self.tick_temperature();

        self.ambient_temperature_delta = self.ambient_temperature() - ambient_temperature_before;
        self.temperature_delta = self.temperature() - temperature_before;
        self.energy_remaining_delta = self.energy_remaining() - energy_remaining_before;

        self.time_alive += self.tick_resolution();

        Ok(())
    }

    /// Is the fire currently burning? Returns `true` if any items in the fire are currently burning, else `false`.
    pub fn is_alive(&self) -> bool {
        self.items
            .iter()
            .any(|x| x.burned_state == BurnedState::Burning)
    }

    /// Does the fire have fresh items?
    ///
    /// **Warning**: This will return true if the fire has burned out.
    pub fn has_fresh_items(&self) -> bool {
        self.items
            .iter()
            .any(|x| x.burned_state == BurnedState::Fresh)
    }

    /// Tick `count` times
    pub fn tick_multiple(&mut self, count: u32) -> Result<(), FireError> {
        for _ in 0..count {
            self.tick()?;
        }

        Ok(())
    }

    /// Tick for `time` time. Will always tick for greater than or equal to `time`. If [`tick_resolution`](Self::tick_resolution()) is too high, this will lead to great inaccuracy.
    pub fn tick_time(&mut self, time: f64) -> Result<(), FireError> {
        self.tick_multiple(f64::ceil(time / self.tick_resolution()) as u32)?;

        Ok(())
    }

    /// Update the temperature of the entire fire for one tick, depending on [Self::tick_time]. The temperature will jump rapidly toward the target when it's far from the it, but be asymptotic toward it as it gets close. If the number of burning items becomes zero, set the fire's temperature to the ambient temperature. The temperature moves more quickly if the fire has less thermal inertia (energy remaining).
    fn tick_temperature(&mut self) {
        if !self.items.is_empty() {
            let target_temperature = self.target_temperature();
            let temperature_difference = target_temperature - self.temperature;
            self.temperature = self.temperature()
                + ((temperature_difference / (50.0/* * self.energy_remaining() THIS IS BAD */))
                    * self.tick_resolution());
        } else {
            self.temperature = self.ambient_temperature();
        }
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
    fn tick_items(&mut self) {
        // TODO: Get rid of the clone() call here for efficiency. This may be possible through std's Cell, or clever references.
        for (i, item) in self.items.clone().into_iter().enumerate() {
            if item.burned_state == BurnedState::Fresh {
                *self.items.get_mut(i).unwrap() = self.heat_item_tick(item);
            } else if item.burned_state == BurnedState::Burning {
                *self.items.get_mut(i).unwrap() = self.burn_item_tick(item);
            }
        }

        // Delete items that have been spent.
        self.items.retain(|x| x.burned_state != BurnedState::Spent);
    }

    /// Tick an unburning item. Items heat up faster if the fire is hotter.
    fn heat_item_tick(&self, mut item: BurningItem) -> BurningItem {
        if self.temperature() >= item.fuel.minimum_activation_temperature {
            // Increase activation progress if the fire temperature is above the minimum activation temperature of the item.
            *item.activation_progress.as_mut().unwrap() +=
                self.temperature() * 0.005 * self.tick_resolution();
        } else {
            // Decay the item's activation progress if the fire temperature is below the minimum activation temperature of the item.
            *item.activation_progress.as_mut().unwrap() -= ((item.fuel.burn_temperature
                - self.ambient_temperature())
                * item.activation_percentage())
                * 0.03
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
    fn burn_item_tick(&self, mut item: BurningItem) -> BurningItem {
        item.remaining_energy -= self.temperature() * 0.001 * self.tick_resolution();

        // The item burns out to spent state if it runs out of potential energy.
        if item.remaining_energy <= 0.0 {
            item.burned_state = BurnedState::Spent;
            item.remaining_energy = 0.0;
        }

        // The item burns out to fresh state if below activation temperature.
        if self.temperature() < item.fuel.minimum_activation_temperature {
            item.burned_state = BurnedState::Fresh;
            item.activation_progress = Some(0.0);
        }

        item
    }
}

/// An error with [`Fire`]
#[derive(Clone, Copy, Error, Debug)]
pub enum FireError {
    #[error("Can not tick the fire after it has died.")]
    TickAfterDead,
}

/// A crafting recipe
#[derive(Debug, Clone)]
pub struct Recipe {
    /// The ingredients for the recipe
    ///
    /// # Element fields
    /// * `0` - The item id
    /// * `1` - The item count
    pub ingredients: Vec<(ItemId, u32)>,

    /// The products of the recipe
    ///
    /// # Element fields
    /// * `0` - The item id
    /// * `1` - The item count
    pub products: Vec<(ItemId, u32)>,

    /// The amount of time it takes to craft the recipe
    pub craft_time: f64,
}

/// A set of crafting recipes
pub struct RecipeSet {
    recipes: Vec<Recipe>,
}

impl RecipeSet {
    /// Create a new RecipeSet
    pub fn new() -> Self {
        RecipeSet {
            recipes: Vec::new(),
        }
    }

    /// Add a recipe
    pub fn push(&mut self, recipe: Recipe) {
        self.recipes.push(recipe);
    }

    /// Fetch a reference to all recipes
    pub fn all(&self) -> &Vec<Recipe> {
        &self.recipes
    }

    /// Find recipes with a specific product
    pub fn filter_product(&self, product: ItemId) -> Vec<&Recipe> {
        self.recipes
            .iter()
            .filter(|x| x.products.iter().any(|x| x.0 == product))
            .collect()
    }
}
