use std::collections::HashMap;

use crate::math::{BoundedFloat, BoundedFloatError};

use super::*;

/// An inventory of items
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Inventory {
    /// The type of item held, and the number of that specific item held
    items: HashMap<ItemId, u32>,
    /// The inventory's used capacity in grams. Bounded to a maximum and a minimum. The minimum is usually `0.0` by default (and as of now, cannot be changed).
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

    /// Get the inventory's used capacity in grams
    pub fn used_capacity(&self) -> BoundedFloat {
        self.used_capacity
    }

    /// Set the inventory's capacity in grams
    ///
    /// # Returns
    /// * [`Ok`]\([`Self`]) - The inventory with the a new max capacity set
    /// * [`Err`]\([`InvalidBounds`](BoundedFloatError::InvalidBounds)) - The max capacity was set below `0.0`.
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
            return Err(InventoryError::NoAvailableCapacity(
                item,
                count,
                self.used_capacity,
            ));
        }

        // Insert the item
        self.used_capacity += mass_of_insertion;
        *self.items.entry(item).or_default() += count;

        Ok(())
    }

    /// Take 1 `item` from the inventory, removing it in-place.
    ///
    /// # Returns
    /// * [`Err`]
    ///     * [`InventoryError::NotEnough`] - if not enough of the item exist in the inventory
    ///     * [`InventoryError::NotFound`] - if no record of the item exists in the inventory
    pub fn take_one(&mut self, item: ItemId) -> Result<(), InventoryError> {
        self.take_amount(item, 1)
    }

    /// Take `count` `item`s from the inventory, removing them in-place.
    ///
    /// # Returns
    /// * [`Err`]
    ///     * [`InventoryError::NotEnough`] - if not enough of the item exist in the inventory
    ///     * [`InventoryError::NotFound`] - if no record of the item exists in the inventory
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
    /// * [`Ok`] - The number of items taken
    /// * [`Err`]
    ///     * [`InventoryError::NotFound`] - if a record of the item does not exist in the inventory
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

    /// Does the inventory contain __all of__ at least each amount of item in `wanted_items`?
    ///
    /// # Returns
    /// * `true` - __All__ items are contained in the inventory.
    /// * `false` - Some of the items are missing.
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
    /// * [`Err`]
    ///     * [`NotEnoughVec`](InventoryError::NotEnoughVec) - The inventory does not contain enough items to be taken. __No items have been removed.__
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

    pub fn burned_state(&self) -> BurnedState {
        self.burned_state
    }

    pub fn set_burned_state(&mut self, value: BurnedState) {
        self.burned_state = value;
    }

    pub fn remaining_energy(&self) -> f64 {
        self.remaining_energy
    }

    pub fn set_remaining_energy(&mut self, value: f64) {
        self.remaining_energy = value;
    }

    pub fn activation_progress(&self) -> Option<f64> {
        self.activation_progress
    }

    pub fn activation_progress_mut(&mut self) -> &mut Option<f64> {
        &mut self.activation_progress
    }

    pub fn set_activation_progress(&mut self, value: Option<f64>) {
        self.activation_progress = value;
    }

    pub fn item(&self) -> &Item {
        &self.item
    }

    pub fn fuel(&self) -> &FuelItem {
        &self.fuel
    }
}
