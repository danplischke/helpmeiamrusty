//! # Threads and `'static` (reference solution)
//!
//! - `thread::spawn<F: FnOnce() -> T + Send + 'static, T: Send + 'static>`:
//!   nothing guarantees the thread finishes before the caller's data is
//!   gone, so the closure may not borrow it (E0521, "borrowed data escapes
//!   outside of function").
//! - `thread::scope` joins all its threads before returning, so `'static`
//!   isn't needed. A scoped thread can even return a `&str` into `docs`.
//!   Collect the handles before joining, or each thread is joined before
//!   the next one starts.
//! - `spawn_count` hands back a `JoinHandle` that outlives the call, so it
//!   has to use `thread::spawn` and give the thread owned copies. For big
//!   shared data, an `Arc<str>` or `Arc<[T]>` avoids copying it per thread.
//! - `max_by_key` returns the *last* of equal maxima; the `fold` keeps the
//!   first.

use std::thread::{self, JoinHandle};

/// The longest line of `doc`, the first one on ties; "" if there's none.
fn longest_line(doc: &str) -> &str {
    doc.lines().fold(
        "",
        |best, line| if line.len() > best.len() { line } else { best },
    )
}

/// The longest line (in bytes) of each document, the first one on ties,
/// computed on one thread per document. Results come back in document
/// order as slices of the documents; an empty document gives "".
fn longest_lines(docs: &[String]) -> Vec<&str> {
    thread::scope(|s| {
        let handles: Vec<_> = docs
            .iter()
            .map(|doc| s.spawn(move || longest_line(doc)))
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().expect("a worker thread panicked"))
            .collect()
    })
}

/// Starts a background thread that counts the lines containing `needle`
/// and returns its handle. The thread may still be running after the
/// caller's data is gone, so it can't borrow `lines` or `needle`.
fn spawn_count(lines: &[&str], needle: &str) -> JoinHandle<usize> {
    let lines: Vec<String> = lines.iter().map(|line| line.to_string()).collect();
    let needle = needle.to_string();
    thread::spawn(move || lines.iter().filter(|line| line.contains(&needle)).count())
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
