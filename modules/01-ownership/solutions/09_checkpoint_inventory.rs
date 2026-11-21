//! # Checkpoint: an inventory (reference solution)
//!
//! - `match self.items.iter_mut().find(..) { Some(item) => .., None => self.items.push(..) }`
//!   compiles: in the `None` arm no borrow from the scrutinee is alive.
//! - `take` finds a *position* first, so it can both mutate the item and
//!   later `remove(pos)` once that borrow has ended. `ok_or(..)?` turns the
//!   `Option` into an early error return, with no `unwrap`.
//! - `max_by_key` returns the **last** of several equal maxima. Searching
//!   the reversed iterator makes it return the first one.
//! - `rename` checks everything before changing anything, so failure leaves
//!   the inventory untouched.
//! - `mem::take` hands the log over without cloning it.

#[derive(Debug, PartialEq)]
struct Item {
    name: String,
    qty: u32,
}

#[derive(Debug, PartialEq)]
enum InventoryError {
    UnknownItem,
    NotEnough { available: u32 },
}

#[derive(Debug, Default)]
struct Inventory {
    items: Vec<Item>,
    audit: Vec<String>,
}

impl Inventory {
    /// Adds `qty` of `name`, creating the item if it's new.
    /// Audits "restock <name> +<qty>".
    fn restock(&mut self, name: &str, qty: u32) {
        match self.items.iter_mut().find(|item| item.name == name) {
            Some(item) => item.qty += qty,
            None => self.items.push(Item {
                name: name.to_string(),
                qty,
            }),
        }
        self.audit.push(format!("restock {name} +{qty}"));
    }

    /// Removes `qty` of `name` and returns how many are left. An item that
    /// reaches zero is removed entirely. Audits "take <name> -<qty>", but
    /// only if it succeeds.
    fn take(&mut self, name: &str, qty: u32) -> Result<u32, InventoryError> {
        let pos = self
            .items
            .iter()
            .position(|item| item.name == name)
            .ok_or(InventoryError::UnknownItem)?;
        let item = &mut self.items[pos];
        if item.qty < qty {
            return Err(InventoryError::NotEnough {
                available: item.qty,
            });
        }
        item.qty -= qty;
        let left = item.qty;
        if left == 0 {
            self.items.remove(pos);
        }
        self.audit.push(format!("take {name} -{qty}"));
        Ok(left)
    }

    /// The item with the highest quantity; the earliest-added one on ties.
    fn most_stocked(&self) -> Option<&Item> {
        self.items.iter().rev().max_by_key(|item| item.qty)
    }

    /// Renames an item. Returns false, changing nothing, if `old` doesn't
    /// exist or `new` already does.
    fn rename(&mut self, old: &str, new: &str) -> bool {
        if self.items.iter().any(|item| item.name == new) {
            return false;
        }
        match self.items.iter_mut().find(|item| item.name == old) {
            Some(item) => {
                item.name = new.to_string();
                true
            }
            None => false,
        }
    }

    /// Hands over the audit log, leaving an empty one.
    fn take_audit(&mut self) -> Vec<String> {
        std::mem::take(&mut self.audit)
    }
}

fn main() {
    let mut inventory = Inventory::default();
    inventory.restock("bolts", 40);
    inventory.restock("nuts", 25);
    println!("{:?}", inventory.take("nuts", 30));
    println!("{:?}", inventory.take("nuts", 25));
    inventory.rename("bolts", "hex bolts");
    println!("{:?}", inventory.most_stocked());
    println!("{:?}", inventory.take_audit());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stocked() -> Inventory {
        let mut inventory = Inventory::default();
        inventory.restock("bolts", 40);
        inventory.restock("nuts", 25);
        inventory.restock("washers", 40);
        inventory
    }

    #[test]
    fn restocking_adds_up() {
        let mut inventory = stocked();
        inventory.restock("nuts", 5);
        assert_eq!(inventory.items.len(), 3);
        assert_eq!(inventory.take("nuts", 10), Ok(20));
    }

    #[test]
    fn taking() {
        let mut inventory = stocked();
        assert_eq!(inventory.take("bolts", 15), Ok(25));
        assert_eq!(
            inventory.take("bolts", 30),
            Err(InventoryError::NotEnough { available: 25 })
        );
        assert_eq!(inventory.take("gears", 1), Err(InventoryError::UnknownItem));
    }

    #[test]
    fn empty_items_disappear() {
        let mut inventory = stocked();
        assert_eq!(inventory.take("nuts", 25), Ok(0));
        assert_eq!(inventory.take("nuts", 1), Err(InventoryError::UnknownItem));
        assert_eq!(inventory.items.len(), 2);
    }

    #[test]
    fn most_stocked_prefers_the_earliest_on_ties() {
        let inventory = stocked();
        let top = inventory.most_stocked().map(|item| item.name.as_str());
        assert_eq!(top, Some("bolts"));
        assert_eq!(Inventory::default().most_stocked(), None);
    }

    #[test]
    fn renaming() {
        let mut inventory = stocked();
        assert!(inventory.rename("nuts", "hex nuts"));
        assert!(!inventory.rename("nuts", "anything"));
        assert!(!inventory.rename("bolts", "washers"));
        assert_eq!(inventory.take("hex nuts", 5), Ok(20));
        assert_eq!(inventory.take("bolts", 1), Ok(39));
    }

    #[test]
    fn audit_log_is_handed_over() {
        let mut inventory = stocked();
        inventory.take("bolts", 1).ok();
        inventory.take("bolts", 100).ok();
        assert_eq!(
            inventory.take_audit(),
            [
                "restock bolts +40",
                "restock nuts +25",
                "restock washers +40",
                "take bolts -1",
            ]
        );
        assert!(inventory.take_audit().is_empty());
    }
}
