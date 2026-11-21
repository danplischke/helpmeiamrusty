//! # The quick option: `Box<dyn Error>`
//!
//! For scripts, prototypes and `main`, a boxed trait object accepts any
//! error through `?`, and a `String` or `&str` message through `.into()`.
//! `main` itself may return `Result<(), E>`. But a plain `Box<dyn Error>`
//! can't cross threads, which is why the usual spelling is
//! `Box<dyn Error + Send + Sync>`. This parallel summer doesn't compile.
//! Fix it without changing what it does.

use std::error::Error;
use std::thread;

type BoxError = Box<dyn Error>;

/// Adds up the comma-separated numbers in `chunk` (spaces around them are
/// fine). A blank chunk is the error "empty chunk"; a bad number is the
/// `ParseIntError`.
fn sum_chunk(chunk: &str) -> Result<u64, BoxError> {
    if chunk.trim().is_empty() {
        return Err("empty chunk");
    }
    let mut total = 0;
    for field in chunk.split(',') {
        total += field.trim().parse::<u64>()?;
    }
    Ok(total)
}

/// Sums each chunk on its own thread, then adds up the results. If several
/// chunks fail, returns the error of the first one, in input order. A
/// worker that panics becomes the error "worker panicked".
fn sum_parallel(chunks: &[&str]) -> Result<u64, BoxError> {
    thread::scope(|s| {
        let workers: Vec<_> = chunks
            .iter()
            .map(|chunk| s.spawn(move || sum_chunk(chunk)))
            .collect();
        let mut total = 0;
        for worker in workers {
            total += worker.join()??;
        }
        Ok(total)
    })
}

fn main() -> Result<(), BoxError> {
    let total = sum_parallel(&["1, 2, 3", "10,20", "100"])?;
    println!("{total}");
    if let Err(e) = sum_parallel(&["1,2", " ", "x"]) {
        println!("{e}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::ParseIntError;

    fn assert_send_sync<T: Send + Sync>(_: &T) {}

    #[test]
    fn sums_chunks() {
        assert_eq!(sum_chunk("1, 2, 3").unwrap(), 6);
        assert_eq!(sum_chunk("42").unwrap(), 42);
        assert_eq!(sum_chunk(" ").unwrap_err().to_string(), "empty chunk");
        let err = sum_chunk("1,two").unwrap_err();
        assert!(err.downcast_ref::<ParseIntError>().is_some());
        assert_send_sync(&err);
    }

    #[test]
    fn sums_in_parallel() {
        assert_eq!(sum_parallel(&["1, 2, 3", "10,20", "100"]).unwrap(), 136);
        assert_eq!(sum_parallel(&[]).unwrap(), 0);
    }

    #[test]
    fn reports_the_first_failing_chunk() {
        let err = sum_parallel(&["1,2", "", "x"]).unwrap_err();
        assert_eq!(err.to_string(), "empty chunk");
        let err = sum_parallel(&["1,2", "x", ""]).unwrap_err();
        assert!(err.downcast_ref::<ParseIntError>().is_some());
    }

    #[test]
    fn a_panicking_worker_is_an_error() {
        // u64::MAX + 1 overflows, which panics in a debug build.
        let err = sum_parallel(&["1", "18446744073709551615, 1"]).unwrap_err();
        assert_eq!(err.to_string(), "worker panicked");
    }
}
