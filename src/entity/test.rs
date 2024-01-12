#![cfg(test)]
use super::*;

#[test]
fn inventory_no_available_capacity() {
    let mut inventory = Inventory::new(100.0);
    inventory.insert(Twig, 3).unwrap();
    let lhs = inventory.insert(Twig, 2).unwrap_err();
    assert!(
        matches!(
            lhs,
            InventoryError::NoAvailableCapacity {
                item: _,
                count: _,
                used_capacity: _,
                max_capacity: _
            }
        ),
        "{lhs:?}\n{lhs}"
    );
}

#[test]
fn inventory_no_capacity() {
    let mut inventory = Inventory::new(100.0);
    inventory.insert(Twig, 3).unwrap();
    let lhs = inventory.insert(Twig, 8).unwrap_err();
    assert!(
        matches!(lhs, InventoryError::NoCapacity(_, _, _)),
        "{lhs:?}\n{lhs}"
    );
}
