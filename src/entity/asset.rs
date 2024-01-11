use thiserror::Error;

use super::*;

impl ItemId {
    /// Get an item's base data from asset definitions.
    fn item(&self) -> Item {
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
    fn fuel(&self) -> Option<FuelItem> {
        match self {
            Twig => Some(FuelItem {
                burn_energy: 10.0,
                burn_temperature: 873.15,
                activation_coefficient: 0.50,
                minimum_activation_temperature: 533.15,
            }),
            SmallStick => Some(FuelItem {
                burn_energy: 500.0,
                burn_temperature: 873.15,
                activation_coefficient: 0.50,
                minimum_activation_temperature: 533.15,
            }),
            MediumStick => Some(FuelItem {
                burn_energy: 1000.0,
                burn_temperature: 873.15,
                activation_coefficient: 0.50,
                minimum_activation_temperature: 533.15,
            }),
            LargeStick => Some(FuelItem {
                burn_energy: 2000.0,
                burn_temperature: 873.15,
                activation_coefficient: 0.50,
                minimum_activation_temperature: 533.15,
            }),
            MediumLog => Some(FuelItem {
                burn_energy: 3500.0,
                burn_temperature: 873.15,
                activation_coefficient: 0.50,
                minimum_activation_temperature: 533.15,
            }),
            LargeLog => Some(FuelItem {
                burn_energy: 5000.0,
                burn_temperature: 873.15,
                activation_coefficient: 0.50,
                minimum_activation_temperature: 533.15,
            }),
            Leaves => Some(FuelItem {
                burn_energy: 100.0,
                burn_temperature: 773.15,
                activation_coefficient: 1.5,
                minimum_activation_temperature: 673.15,
            }),
        }
    }

    /// Get an item's weapon data from asset definitions. Returns [`None`] if the item is not a [`WeaponItem`].
    fn weapon(&self) -> Option<WeaponItem> {
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

/// Error obtaining an asset from asset definitions
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Error)]
pub enum AssetError {
    /// Asset not found
    #[error("Asset not found: {0:?}")]
    NotFound(ItemId),
}

impl From<ItemId> for Item {
    fn from(value: ItemId) -> Self {
        value.item()
    }
}

impl TryFrom<ItemId> for FuelItem {
    type Error = AssetError;

    fn try_from(value: ItemId) -> Result<Self, Self::Error> {
        value.fuel().ok_or(AssetError::NotFound(value))
    }
}

impl TryFrom<ItemId> for WeaponItem {
    type Error = AssetError;

    fn try_from(value: ItemId) -> Result<Self, Self::Error> {
        value.weapon().ok_or(AssetError::NotFound(value))
    }
}
