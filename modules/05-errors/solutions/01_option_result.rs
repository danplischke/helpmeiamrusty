//! # Between Option and Result (reference solution)
//!
//! - `ok_or(err)` builds `err` even when the value is there. That's fine
//!   for a cheap value like `NoSuchEmployee(id)`. When the error allocates
//!   (`NoManager(name.clone())`), use `ok_or_else(|| ..)` so it's only built
//!   on the failure path. Clippy nudges the other way too: a closure around
//!   a cheap value is `unnecessary_lazy_evaluations`.
//! - In a function returning `Option`, `?` unwraps each `Option` step, and
//!   `.ok()` turns a `Result` into an `Option` so `?`-chains keep going.
//! - `bool::then` builds an `Option` lazily, and `transpose` flips
//!   `Option<Result<..>>` into the `Result<Option<..>>` the signature wants.
//!   An explicit `if field.is_empty() { Ok(None) } else { .. }` is just as
//!   good; reach for `transpose` when the `Option` is already in hand.

use std::num::ParseIntError;

#[derive(Debug, PartialEq)]
struct Employee {
    id: u32,
    name: String,
    manager: Option<u32>,
    desk: Option<String>,
}

#[derive(Debug, PartialEq)]
enum LookupError {
    NoSuchEmployee(u32),
    NoManager(String),
}

/// The employee with this id, or `NoSuchEmployee(id)`.
fn find(staff: &[Employee], id: u32) -> Result<&Employee, LookupError> {
    staff
        .iter()
        .find(|e| e.id == id)
        .ok_or(LookupError::NoSuchEmployee(id))
}

/// The manager of employee `id`. Errors: `NoSuchEmployee` if `id` is
/// unknown (or the manager's id is), `NoManager(<employee's name>)` if the
/// employee has no manager.
fn manager_of(staff: &[Employee], id: u32) -> Result<&Employee, LookupError> {
    let employee = find(staff, id)?;
    let manager = employee
        .manager
        .ok_or_else(|| LookupError::NoManager(employee.name.clone()))?;
    find(staff, manager)
}

/// The floor of an employee's desk: desk "4-17" is on floor 4. `None` if
/// they have no desk, or the desk has no '-' or no number before it.
fn desk_floor(employee: &Employee) -> Option<u32> {
    let (floor, _) = employee.desk.as_deref()?.split_once('-')?;
    floor.parse().ok()
}

/// Parses the manager column of an import file. A blank field (empty or
/// only whitespace) means "no manager"; anything else must be a number,
/// surrounding whitespace allowed.
fn parse_manager_field(field: &str) -> Result<Option<u32>, ParseIntError> {
    let field = field.trim();
    (!field.is_empty()).then(|| field.parse()).transpose()
}

fn staff() -> Vec<Employee> {
    let employee = |id, name: &str, manager, desk: Option<&str>| Employee {
        id,
        name: name.to_string(),
        manager,
        desk: desk.map(str::to_string),
    };
    vec![
        employee(1, "Grace", None, Some("9-1")),
        employee(2, "Alan", Some(1), Some("4-17")),
        employee(3, "Ada", Some(2), None),
        employee(4, "Linus", Some(99), Some("lobby")),
    ]
}

fn main() {
    let staff = staff();
    println!("{:?}", manager_of(&staff, 3).map(|m| &m.name));
    println!("{:?}", manager_of(&staff, 1));
    println!("{:?}", desk_floor(&staff[1]));
    println!("{:?}", parse_manager_field(" 7 "));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finding() {
        let staff = staff();
        assert_eq!(find(&staff, 3).map(|e| e.name.as_str()), Ok("Ada"));
        assert_eq!(find(&staff, 7), Err(LookupError::NoSuchEmployee(7)));
    }

    #[test]
    fn managers() {
        let staff = staff();
        assert_eq!(manager_of(&staff, 3).map(|e| e.id), Ok(2));
        assert_eq!(manager_of(&staff, 2).map(|e| e.id), Ok(1));
        assert_eq!(
            manager_of(&staff, 1),
            Err(LookupError::NoManager("Grace".to_string()))
        );
        assert_eq!(manager_of(&staff, 8), Err(LookupError::NoSuchEmployee(8)));
        assert_eq!(manager_of(&staff, 4), Err(LookupError::NoSuchEmployee(99)));
    }

    #[test]
    fn desk_floors() {
        let staff = staff();
        let floors: Vec<_> = staff.iter().map(desk_floor).collect();
        assert_eq!(floors, [Some(9), Some(4), None, None]);
        let mut odd = staff.into_iter().next().unwrap();
        odd.desk = Some("x-3".to_string());
        assert_eq!(desk_floor(&odd), None);
    }

    #[test]
    fn manager_fields() {
        assert_eq!(parse_manager_field("7"), Ok(Some(7)));
        assert_eq!(parse_manager_field(" 12 "), Ok(Some(12)));
        assert_eq!(parse_manager_field(""), Ok(None));
        assert_eq!(parse_manager_field("   "), Ok(None));
        let err = parse_manager_field("seven").unwrap_err();
        assert_eq!(err.to_string(), "invalid digit found in string");
    }
}
