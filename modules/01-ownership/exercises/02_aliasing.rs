//! # Shared XOR mutable
//!
//! At any moment a value can have many `&` borrows *or* one `&mut` borrow,
//! never both. Each function breaks that rule once. Fix them by reordering,
//! shortening a borrow, or copying out a `Copy` value. No `.clone()`.

/// Appends a copy of the first element, then returns that element.
/// `v` is never empty.
fn repeat_first(v: &mut Vec<i32>) -> i32 {
    let first = &v[0];
    v.push(*first);
    *first
}

/// For every number over 10, appends half of it. Only the numbers that were
/// there to begin with count (the appended halves aren't halved again).
fn append_halves(v: &mut Vec<u32>) {
    for x in v.iter() {
        if *x > 10 {
            v.push(*x / 2);
        }
    }
}

struct Library {
    books: Vec<String>,
    loans: u32,
}

impl Library {
    fn first_title(&self) -> &str {
        &self.books[0]
    }

    fn record_loan(&mut self) {
        self.loans += 1;
    }
}

/// Lends the first book: records the loan and returns "lent <title>".
fn lend_first(lib: &mut Library) -> String {
    let title = lib.first_title();
    lib.record_loan();
    format!("lent {title}")
}

fn main() {
    let mut v = vec![7, 1];
    println!("{} {v:?}", repeat_first(&mut v));
    let mut nums = vec![4, 30, 12];
    append_halves(&mut nums);
    println!("{nums:?}");
    let mut lib = Library {
        books: vec!["Dune".to_string()],
        loans: 0,
    };
    println!("{} ({} loans)", lend_first(&mut lib), lib.loans);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeats_first() {
        let mut v = vec![7, 1];
        assert_eq!(repeat_first(&mut v), 7);
        assert_eq!(v, [7, 1, 7]);
    }

    #[test]
    fn appends_halves_of_original_values() {
        let mut v = vec![4, 30, 12];
        append_halves(&mut v);
        assert_eq!(v, [4, 30, 12, 15, 6]);
    }

    #[test]
    fn lends() {
        let mut lib = Library {
            books: vec!["Dune".to_string(), "Emma".to_string()],
            loans: 0,
        };
        assert_eq!(lend_first(&mut lib), "lent Dune");
        assert_eq!(lib.loans, 1);
    }
}
