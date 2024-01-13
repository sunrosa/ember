use enum_as_inner::EnumAsInner;

use super::*;

/// In order to complete the craft immediately, call [`complete()`](Self::complete()), and it will tick the fire accordingly. If you have limited time to await the craft, call [`progress()`](Self::progress()) to progress the craft by a specified amount of time.
#[derive(Clone, Debug, PartialEq)]
pub struct InProgressCraft {
    /// The ingredients of the recipe
    pub(super) ingredients: &'static Vec<(ItemId, u32)>,
    /// The products of the recipe
    pub(super) products: &'static Vec<(ItemId, u32)>,
    /// The total time the recipe takes
    pub(super) recipe_time: f64,
    /// The amount of time that remains until the recipe is completed
    pub(super) time_remaining: f64,
}

// This really, really reminds me of Futures lol. I forgot what this process is called. "Make invalid states unrepresentable" or some shit. I think it's the Finite-State-Machine pattern. I like it a fucking hell of a lot though :3
impl InProgressCraft {
    // TODO move this to Player as a field.
    /// The coefficient of uncrafting. Higher is faster.
    const UNCRAFT_MULTIPLIER: f64 = 4.0;

    /// Finish off the craft now, ticking the fire for however long the craft has remaining, returning the products. This method takes ownership and drops its receiver.
    ///
    /// # Returns
    /// * [`Ok`] - The craft successfully completed. Contained are the products.
    /// * [`Err`]\([`BurntOut`](FireError::BurntOut)) - The fire burnt out while crafting.
    pub fn complete(self, fire: &mut Fire) -> Result<&'static Vec<(ItemId, u32)>, FireError> {
        fire.tick_time(self.time_remaining)?;
        Ok(self.products)
    }

    /// Progress the craft by `time` time, "polling" it. This method will take only the time necessary to finish the craft, and not the entire amount of time specified. Because this method takes ownership of its receiver, you will have to use its returned [`CraftResult`] exclusively.
    ///
    /// # Returns
    /// * [`Ok`]
    ///     * [`Ready`](CraftResult::Ready) - The craft has completed. Contained are the products.
    ///     * [`Pending`](CraftResult::Pending) - There is still more time needed to complete the task.
    /// * [`Err`]\([`BurntOut`](FireError::BurntOut)) - The fire burnt out while crafting.
    pub fn progress(mut self, fire: &mut Fire, max_time: f64) -> Result<CraftResult, FireError> {
        if max_time >= self.time_remaining {
            // Ready
            fire.tick_time(self.time_remaining)?;
            return Ok(CraftResult::Ready(self.products));
        } else {
            // Pending
            fire.tick_time(max_time)?;
            self.time_remaining -= max_time;
            Ok(CraftResult::Pending(self))
        }
    }

    /// Reverse and cancel the craft and return its ingredients to be given back to the player. Uncrafts are 4x as fast as crafts. This will be even faster if the player was early in the craft. This method drops its receiver.
    ///
    /// # Returns
    /// * [`Ok`] - The uncraft successfully completed. Contained are the ingredients.
    /// * [`Err`]\([`BurntOut`](FireError::BurntOut)) - The fire burnt out while crafting.
    pub fn cancel(self, fire: &mut Fire) -> Result<&'static Vec<(ItemId, u32)>, FireError> {
        fire.tick_time(self.uncraft_time())?;
        Ok(self.ingredients)
    }

    /// Reverse a craft (uncraft) by `time` time, "polling" it. This method will take only the time necessary to finish the uncraft, and not the entire amount of time specified. Because this method takes ownership of its receiver, you will have to use its returned [`CraftResult`] exclusively.
    ///
    /// # Returns
    /// * [`Ok`]
    ///     * [`Ready`](CraftResult::Ready) - The uncraft has completed. Contained are the ingredients.
    ///     * [`Pending`](CraftResult::Pending) - There is still more time needed to complete the uncraft.
    /// * [`Err`]\([`BurntOut`](FireError::BurntOut)) - The fire burnt out while crafting.
    pub fn progress_cancel(
        mut self,
        fire: &mut Fire,
        max_time: f64,
    ) -> Result<CraftResult, FireError> {
        let time_left = self.uncraft_time();

        if max_time >= time_left {
            // Ready
            fire.tick_time(time_left)?;
            return Ok(CraftResult::Ready(self.ingredients));
        } else {
            // Pending
            fire.tick_time(max_time)?;
            self.time_remaining += max_time * Self::UNCRAFT_MULTIPLIER; // Critically, this INCREASES the time remaining
            Ok(CraftResult::Pending(self))
        }
    }

    /// Calculate the time necessary to reverse craft a [`Self`]. Uncrafting is 4x as fast as crafting.
    fn uncraft_time(&self) -> f64 {
        (self.recipe_time - self.time_remaining) / Self::UNCRAFT_MULTIPLIER
    }
}

/// The result of "polling" a crafting process
#[derive(Debug, Clone, EnumAsInner)]
pub enum CraftResult {
    /// The craft is ready. Contained are the item products of the recipe.
    Ready(&'static Vec<(ItemId, u32)>),
    /// The craft is still pending. Contained is the in-progress craft to be "polled" again.
    Pending(InProgressCraft),
}

/// Result of checking to see if there are enough items in an inventory to craft a recipe
pub enum EnoughItems {
    /// There are enough items.
    Enough,
    /// The following items are missing. If the inventory partially contains an item necessary, they will be subtracted from this difference.
    Missing(Vec<(ItemId, u32)>),
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

impl Default for RecipeSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn init() -> (Fire, Player) {
        let (fire, player) = (Fire::init(), Player::init());
        assert_eq!(
            fire.time_alive(),
            0.0,
            "The fire should start with 0.0 time passed."
        );

        (fire, player)
    }

    #[test]
    fn no_recipe() {
        let (_, mut player) = init();
        let craft = player.craft(ItemId::Twig);

        assert_eq!(craft, Err(CraftError::NoRecipe(ItemId::Twig)));
    }

    #[test]
    fn missing_ingredients() {
        let (_, mut player) = init();
        let craft = player.craft(ItemId::SmallBundle);

        assert_eq!(
            craft,
            Err(CraftError::MissingIngredients(vec![(
                ItemId::SmallStick,
                3
            )]))
        );
    }

    #[test]
    fn craft_progress() {
        let (mut fire, mut player) = init();
        player
            .inventory_mut()
            .insert(ItemId::SmallStick, 3)
            .unwrap();
        let mut craft = player.craft(ItemId::SmallBundle).unwrap();

        craft = craft
            .progress(&mut fire, 50.0)
            .unwrap()
            .into_pending()
            .unwrap();

        assert_eq!(fire.time_alive(), 50.0);

        let products = craft
            .progress(&mut fire, 50.0)
            .unwrap()
            .into_ready()
            .unwrap();

        assert_eq!(fire.time_alive(), 100.0);
        assert_eq!(*products, vec![(ItemId::SmallBundle, 1)]);
    }

    #[test]
    fn craft_complete() {
        let (mut fire, mut player) = init();
        player
            .inventory_mut()
            .insert(ItemId::SmallStick, 3)
            .unwrap();
        let products = player
            .craft(ItemId::SmallBundle)
            .unwrap()
            .complete(&mut fire)
            .unwrap();

        assert_eq!(fire.time_alive(), 100.0);
        assert_eq!(*products, vec![(ItemId::SmallBundle, 1)]);
    }

    #[test]
    fn craft_cancel() {
        let (mut fire, mut player) = init();
        player
            .inventory_mut()
            .insert(ItemId::SmallStick, 3)
            .unwrap();
        let mut craft = player.craft(ItemId::SmallBundle).unwrap();

        craft = craft
            .progress(&mut fire, 50.0)
            .unwrap()
            .into_pending()
            .unwrap();

        assert_eq!(fire.time_alive(), 50.0);

        let ingredients = craft.cancel(&mut fire).unwrap();

        assert_eq!(fire.time_alive(), 63.0);
        assert_eq!(*ingredients, vec![(ItemId::SmallStick, 3)]);
    }
    #[test]
    fn craft_progress_cancel() {
        let (mut fire, mut player) = init();
        player
            .inventory_mut()
            .insert(ItemId::SmallStick, 3)
            .unwrap();
        let mut craft = player.craft(ItemId::SmallBundle).unwrap();

        craft = craft
            .progress(&mut fire, 50.0)
            .unwrap()
            .into_pending()
            .unwrap();

        assert_eq!(fire.time_alive(), 50.0);

        craft = craft
            .progress_cancel(&mut fire, 10.0)
            .unwrap()
            .into_pending()
            .unwrap();

        assert_eq!(fire.time_alive(), 60.0);

        let ingredients = craft.cancel(&mut fire).unwrap();

        assert_eq!(fire.time_alive(), 63.0);
        assert_eq!(*ingredients, vec![(ItemId::SmallStick, 3)]);
    }
}
