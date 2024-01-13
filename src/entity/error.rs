use thiserror::Error;

use crate::math::BoundedFloat;

use super::*;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
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
    ///
    /// * `0` - The item id
    #[error("The item {0:?} does not exist in the inventory.")]
    NotFound(ItemId),

    /// Not enough of the item to be taken from the inventory.
    ///
    /// * `0` - The item id
    /// * `1` - The amount of the item in the inventory
    #[error(
        "Not enough of the item {0:?} to take from the inventory. Count {1} are currently available."
    )]
    NotEnough(ItemId, u32),

    /// Could never store that many of the item even if the inventory were empty.
    ///
    /// * `0` - The item id
    /// * `1` - The count of the item trying to be stored
    /// * `2` - The total capacity of the inventory
    #[error("Could never store count {1} of item {0:?}, even when empty.\nTotal capacity: {2}")]
    NoCapacity(ItemId, u32, f64),

    /// The inventory does not have the available capacity to store that many of the item.
    ///
    /// * `0` - The item id
    /// * `1` - The number of items trying to be stored
    /// * `2` - The capacity of the inventory
    #[error("Not enough available capacity to store count {1} of item {0:?}.\nUsed capacity: {}\nTotal capacity: {}\n", .2.current(), .2.max())]
    NoAvailableCapacity(ItemId, u32, BoundedFloat),

    /// The following [`Vec`] of items are missing.
    #[error("The following items are missing: {0:?}")]
    NotEnoughVec(Vec<(ItemId, u32)>),
}

/// An error thrown when trying to construct a [`BurningItem`].
#[derive(Debug, Clone, Copy, Error)]
pub enum BurnItemError {
    /// The item in question is not flammable (or simply lacks needed burn properties in asset definitions).
    #[error("{0:?} is not a flammable item.")]
    NotFlammable(ItemId),
}

/// An error with [`Fire`]
#[derive(Clone, Copy, Error, Debug)]
pub enum FireError {
    #[error("Can not tick the fire after it has died.")]
    BurntOut,
}
