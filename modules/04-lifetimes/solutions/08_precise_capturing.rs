//! # Capture only what you borrow (reference solution)
//!
//! - Edition 2024 made return-position `impl Trait` capture every lifetime
//!   in scope. Before, a lifetime was captured only if it appeared in the
//!   bounds (hence the old `+ '_` idiom). Capturing is usually right:
//!   `header` really borrows `self`.
//! - `row_indices` builds its range up front, so it borrows nothing:
//!   `use<>` lets the caller mutate the sheet while iterating. rustc's note
//!   "this call may capture more lifetimes than intended" points here.
//! - `fields` only needs `line`, but the closure `|c| c == self.separator`
//!   captured `&self` as well. With `use<'t>` that's E0700 ("hidden type
//!   captures lifetime that does not appear in bounds"), and the suggested
//!   `use<'t, '_>` would bring the original problem back. Pass the `char`
//!   by value instead: `line.split(self.separator)`.
//! - `use<..>` is a promise about the body: whatever you leave out, the
//!   returned value must not hold on to.

/// A table of delimited text lines.
struct Sheet {
    separator: char,
    header: Vec<String>,
    rows: Vec<String>,
}

impl Sheet {
    fn new(separator: char, header: &[&str]) -> Self {
        Sheet {
            separator,
            header: header.iter().map(|name| name.to_string()).collect(),
            rows: Vec::new(),
        }
    }

    fn push_row(&mut self, line: &str) {
        self.rows.push(line.to_string());
    }

    /// The trimmed fields of `line`, as slices of `line`.
    fn fields<'t>(&self, line: &'t str) -> impl Iterator<Item = &'t str> + use<'t> {
        line.split(self.separator).map(str::trim)
    }

    /// The row indices as they are now, `0..number of rows`.
    fn row_indices(&self) -> impl Iterator<Item = usize> + use<> {
        0..self.rows.len()
    }

    /// The column names, borrowed from the sheet.
    fn header(&self) -> impl Iterator<Item = &str> {
        self.header.iter().map(String::as_str)
    }

    /// Rewrites row `i` with its fields trimmed. Panics if there's no row `i`.
    fn normalize_row(&mut self, i: usize) {
        let fields: Vec<&str> = self.fields(&self.rows[i]).collect();
        self.rows[i] = fields.join(&self.separator.to_string());
    }
}

fn main() {
    let mut sheet = Sheet::new(';', &["item", "qty"]);
    sheet.push_row(" bolts ; 40");
    sheet.push_row("nuts;25 ");
    for i in sheet.row_indices() {
        sheet.normalize_row(i);
    }
    println!("{:?} {:?}", sheet.header().collect::<Vec<_>>(), sheet.rows);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sheet() -> Sheet {
        let mut sheet = Sheet::new(';', &["item", "qty"]);
        sheet.push_row(" bolts ; 40");
        sheet.push_row("nuts;  25  ");
        sheet
    }

    /// Builds a throwaway `Sheet` and returns the lazy iterator.
    fn csv_fields(line: &str) -> impl Iterator<Item = &str> {
        Sheet::new(',', &[]).fields(line)
    }

    #[test]
    fn splits_fields() {
        let sheet = sheet();
        let fields: Vec<&str> = sheet.fields(" a ;b; ").collect();
        assert_eq!(fields, ["a", "b", ""]);
        assert_eq!(sheet.fields("").count(), 1);
    }

    #[test]
    fn fields_outlive_the_sheet() {
        let line = String::from("x, y ,z");
        let fields: Vec<&str> = csv_fields(&line).collect();
        assert_eq!(fields, ["x", "y", "z"]);
    }

    #[test]
    fn normalizes_every_row() {
        let mut sheet = sheet();
        for i in sheet.row_indices() {
            sheet.normalize_row(i);
        }
        assert_eq!(sheet.rows, ["bolts;40", "nuts;25"]);
    }

    #[test]
    fn row_indices_are_a_snapshot() {
        let mut sheet = sheet();
        let indices = sheet.row_indices();
        sheet.push_row("washers;100");
        assert_eq!(indices.collect::<Vec<_>>(), [0, 1]);
        assert_eq!(sheet.row_indices().count(), 3);
    }

    #[test]
    fn header_borrows_from_the_sheet() {
        let sheet = sheet();
        let names: Vec<&str> = sheet.header().collect();
        assert_eq!(names, ["item", "qty"]);
    }
}
