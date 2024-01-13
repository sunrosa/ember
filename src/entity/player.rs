use crate::math::BoundedFloat;

use super::*;

// TODO Crafting speed stat.
/// The player that plays the game
///
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Player {
    /// The player's hit points
    hit_points: BoundedFloat,
    /// Body temperature in degrees kelvin
    body_temperature: f64,
    /// The player's inventory
    inventory: Inventory,
    craft_speed: f64,
    uncraft_speed: f64,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            hit_points: BoundedFloat::new_zero_min(100.0, 100.0).unwrap(),
            body_temperature: 310.15,
            inventory: Inventory::new(10000.0),
            craft_speed: 1.0,
            uncraft_speed: 4.0,
        }
    }
}

impl Player {
    /// Create a new [`Player`] with customization. See [`init`](Player::init()) to create a [`Player`] with default parameters.
    pub fn new(max_hp: f64, inventory_capacity: f64) -> Self {
        Self {
            hit_points: BoundedFloat::new_zero_min(max_hp, max_hp).unwrap(),
            body_temperature: 310.15,
            inventory: Inventory::new(inventory_capacity),
            craft_speed: 1.0,
            uncraft_speed: 4.0,
        }
    }

    /// The player's speed of crafting
    pub fn craft_speed(&self) -> f64 {
        self.craft_speed
    }

    /// The player's speed of uncrafting
    pub fn uncraft_speed(&self) -> f64 {
        self.uncraft_speed
    }

    /// Deal `hp` damage to the player.
    pub fn damage(&mut self, hp: f64) {
        self.hit_points -= hp;
    }

    /// Heal the player for `hp`.
    pub fn heal(&mut self, hp: f64) {
        self.hit_points += hp;
    }

    /// Get a mutable reference to the player's inventory.
    pub fn inventory_mut(&mut self) -> &mut Inventory {
        &mut self.inventory
    }

    /// Craft an item, if possible, taking the first craftable recipe if there are multiple. This method accounts for all recipes in the global static recipe set, and also for the items in the player's [`inventory`](Self::inventory_mut).
    ///
    /// # Returns
    /// * [`Ok`] - A recipe has been found and is ready to begin making progress.
    /// * [`Err`]
    ///     * [`MissingIngredients`](CraftError::MissingIngredients) - A recipe was found in the global static recipe set, but the player does not have sufficient items with which to craft it.
    ///     * [`NoRecipe`][CraftError::NoRecipe] - No recipe with the matching product was found.
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
                        ingredients: &recipe.ingredients,
                        products: &recipe.products,
                        recipe_time: recipe.craft_time,
                        time_remaining: recipe.craft_time,
                        craft_speed: self.craft_speed(),
                        uncraft_speed: self.uncraft_speed(),
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
