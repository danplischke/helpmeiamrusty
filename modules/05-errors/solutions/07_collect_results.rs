//! # Many results at once (reference solution)
//!
//! - One helper yields a `Result` per row; the importers only differ in
//!   how they consume it. Iterators are lazy, so `collect()` into
//!   `Result<Vec<_>, _>` really stops parsing at the first error.
//! - `Result<T, E>` implements `Sum` and `Product` when `T` does, with the
//!   same short-circuiting. So does `Option`, and `collect()` into
//!   `Option<Vec<T>>` gives `None` at the first `None`.
//! - For "keep the good, report the bad", a `match` that pushes into two
//!   Vecs is clear and allocates once per item. The alternative,
//!   `partition(Result::is_ok)`, leaves you two `Vec<Result<..>>` to
//!   unwrap afterwards.
//! - Don't reach for `.flatten()` or `.filter_map(Result::ok)` here: they
//!   silently drop the errors, which is rarely what an importer wants.

#[derive(Debug, PartialEq)]
struct Entry {
    name: String,
    hours: u32,
}

#[derive(Debug, PartialEq)]
enum RowError {
    /// No ',' on the line, or nothing before it.
    MissingName { line: usize },
    /// What follows the ',' isn't a whole number of hours.
    BadHours { line: usize, text: String },
}

/// Parses one row, "<name>,<hours>", splitting at the first ','. Both parts
/// are trimmed; `text` in `BadHours` is the trimmed hours part. `line` is
/// the row's line number, used in errors.
fn parse_row(line: usize, row: &str) -> Result<Entry, RowError> {
    let (name, hours) = row.split_once(',').ok_or(RowError::MissingName { line })?;
    let name = name.trim();
    if name.is_empty() {
        return Err(RowError::MissingName { line });
    }
    let text = hours.trim();
    let hours = text.parse().map_err(|_| RowError::BadHours {
        line,
        text: text.to_string(),
    })?;
    Ok(Entry {
        name: name.to_string(),
        hours,
    })
}

/// Every non-blank line of `csv`, parsed, with lines numbered from 1.
fn rows(csv: &str) -> impl Iterator<Item = Result<Entry, RowError>> {
    csv.lines()
        .enumerate()
        .filter(|(_, row)| !row.trim().is_empty())
        .map(|(i, row)| parse_row(i + 1, row))
}

/// Parses every non-blank line of `csv`, numbering lines from 1 (blank
/// lines count). All or nothing: on a bad row, returns the first error.
fn import_strict(csv: &str) -> Result<Vec<Entry>, RowError> {
    rows(csv).collect()
}

/// Like `import_strict`, but keeps going: returns the good entries and
/// every error, both in file order.
fn import_lenient(csv: &str) -> (Vec<Entry>, Vec<RowError>) {
    let mut entries = Vec::new();
    let mut errors = Vec::new();
    for row in rows(csv) {
        match row {
            Ok(entry) => entries.push(entry),
            Err(e) => errors.push(e),
        }
    }
    (entries, errors)
}

/// The total hours of a timesheet, or the first error.
fn total_hours(csv: &str) -> Result<u32, RowError> {
    rows(csv).map(|row| row.map(|entry| entry.hours)).sum()
}

fn main() {
    let csv = "ana,8\nbo,seven\n\n,4\ncy, 6\n";
    println!("{:?}", import_strict(csv));
    let (entries, errors) = import_lenient(csv);
    println!("{} good, {errors:?}", entries.len());
    println!("{:?}", total_hours("ana,8\ncy,6"));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, hours: u32) -> Entry {
        Entry {
            name: name.to_string(),
            hours,
        }
    }

    fn bad_hours(line: usize, text: &str) -> RowError {
        RowError::BadHours {
            line,
            text: text.to_string(),
        }
    }

    #[test]
    fn single_rows() {
        assert_eq!(parse_row(1, "ana,8"), Ok(entry("ana", 8)));
        assert_eq!(parse_row(1, " Zoë , 7 "), Ok(entry("Zoë", 7)));
        assert_eq!(parse_row(2, "ana"), Err(RowError::MissingName { line: 2 }));
        assert_eq!(parse_row(3, " ,8"), Err(RowError::MissingName { line: 3 }));
        assert_eq!(parse_row(4, "ana, 8h"), Err(bad_hours(4, "8h")));
        assert_eq!(parse_row(5, "ana,8,9"), Err(bad_hours(5, "8,9")));
        assert_eq!(parse_row(6, "ana,"), Err(bad_hours(6, "")));
    }

    #[test]
    fn strict_imports_stop_at_the_first_error() {
        let csv = "ana,8\n\nbo,7\n";
        assert_eq!(
            import_strict(csv),
            Ok(vec![entry("ana", 8), entry("bo", 7)])
        );
        assert_eq!(import_strict(""), Ok(vec![]));
        let csv = "ana,8\nbo,x\n\n,3\n";
        assert_eq!(import_strict(csv), Err(bad_hours(2, "x")));
    }

    #[test]
    fn lenient_imports_keep_going() {
        let csv = "ana,8\nbo,x\n\n,3\ncy,6";
        let (entries, errors) = import_lenient(csv);
        assert_eq!(entries, [entry("ana", 8), entry("cy", 6)]);
        assert_eq!(
            errors,
            [bad_hours(2, "x"), RowError::MissingName { line: 4 }]
        );
        let (entries, errors) = import_lenient("\n\n");
        assert!(entries.is_empty() && errors.is_empty());
    }

    #[test]
    fn totals() {
        assert_eq!(total_hours("ana,8\nbo,7\n\ncy,1"), Ok(16));
        assert_eq!(total_hours(""), Ok(0));
        assert_eq!(
            total_hours("ana,8\nbo\ncy,y"),
            Err(RowError::MissingName { line: 2 })
        );
    }
}
