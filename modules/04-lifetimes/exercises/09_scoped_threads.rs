//! # Threads and `'static`
//!
//! `std::thread::spawn` needs a `'static` closure: the thread may outlive
//! the caller, so it can't borrow the caller's locals. Move owned data in
//! instead. `std::thread::scope` joins every thread it spawned before it
//! returns, so those threads may borrow, and may even hand borrows back.
//! Implement both functions.

use std::thread::{self, JoinHandle};

/// The longest line (in bytes) of each document, the first one on ties,
/// computed on one thread per document. Results come back in document
/// order as slices of the documents; an empty document gives "".
fn longest_lines(docs: &[String]) -> Vec<&str> {
    todo!("one thread per document in {docs:?}")
}

/// Starts a background thread that counts the lines containing `needle`
/// and returns its handle. The thread may still be running after the
/// caller's data is gone, so it can't borrow `lines` or `needle`.
fn spawn_count(lines: &[&str], needle: &str) -> JoinHandle<usize> {
    todo!("count {needle:?} in {lines:?} in the background")
}

fn main() {
    let docs = vec![
        String::from("fn main() {\n    println!(\"hi\");\n}"),
        String::from("short"),
    ];
    println!("{:?}", longest_lines(&docs));
    let handle = spawn_count(&["error: disk", "ok", "error: net"], "error");
    println!("{:?}", handle.join());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn longest_line_of_each_document() {
        let docs = vec![
            String::from("short\na much longer line\nmid"),
            String::from("tie one\ntie two"),
            String::new(),
        ];
        assert_eq!(longest_lines(&docs), ["a much longer line", "tie one", ""]);
        assert!(longest_lines(&[]).is_empty());
    }

    #[test]
    fn results_point_into_the_documents() {
        let docs = vec![String::from("x\nyy\nz")];
        let longest = longest_lines(&docs);
        let start = docs[0].as_ptr() as usize;
        let inside = start..start + docs[0].len();
        assert!(inside.contains(&(longest[0].as_ptr() as usize)));
    }

    #[test]
    fn background_count_outlives_its_input() {
        let handle = {
            let text = String::from("error: disk\ninfo: ok\nerror: net");
            let lines: Vec<&str> = text.lines().collect();
            spawn_count(&lines, "error")
        }; // `text` and `lines` are gone; the thread must own its data
        assert_eq!(handle.join().unwrap(), 2);
        assert_eq!(spawn_count(&[], "x").join().unwrap(), 0);
    }
}
