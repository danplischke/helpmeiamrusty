//! # Between Option and Result
//!
//! `Option` says *whether*, `Result` says *why not*. Converting is a
//! one-liner each way: `ok_or` / `ok_or_else` attach a reason, `.ok()`
//! drops it, and `transpose` swaps `Option<Result<T, E>>` and
//! `Result<Option<T>, E>`. `?` works on both, but only on the kind the
//! function itself returns. Each body fits in a few lines, without `match`.

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
    todo!("find {id} in {staff:?}")
}

/// The manager of employee `id`. Errors: `NoSuchEmployee` if `id` is
/// unknown (or the manager's id is), `NoManager(<employee's name>)` if the
/// employee has no manager.
fn manager_of(staff: &[Employee], id: u32) -> Result<&Employee, LookupError> {
    todo!("find the manager of {id} in {staff:?}")
}

/// The floor of an employee's desk: desk "4-17" is on floor 4. `None` if
/// they have no desk, or the desk has no '-' or no number before it.
fn desk_floor(employee: &Employee) -> Option<u32> {
    todo!("find the floor of {employee:?}")
}

/// Parses the manager column of an import file. A blank field (empty or
/// only whitespace) means "no manager"; anything else must be a number,
/// surrounding whitespace allowed.
fn parse_manager_field(field: &str) -> Result<Option<u32>, ParseIntError> {
    todo!("parse {field:?}")
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
