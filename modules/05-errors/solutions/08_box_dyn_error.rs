//! # The quick option: `Box<dyn Error>` (reference solution)
//!
//! - `Box<dyn Error + Send + Sync>` has `From` impls for every
//!   `E: Error + Send + Sync`, for `String` and for `&str`, so `?` and
//!   `.into()` cover everything. A bare `Err("..")` is still a `&str`:
//!   only `?` and `.into()` convert.
//! - Without `Send` the error can't leave a worker thread (E0277 at
//!   `spawn`), and `Sync` lets it be shared by reference, as in an `Arc`.
//!   Write both bounds from the start; the alias keeps it short.
//! - `join()` returns `Err(payload)` if the thread panicked. The payload is
//!   a `Box<dyn Any + Send>`, not an error, so it has to be mapped to one.
//! - You can still `downcast_ref::<ParseIntError>()` a boxed error, but
//!   callers who need to match on failures deserve an enum. Box errors for
//!   binaries and prototypes; typed errors for libraries.
//! - `main` returning `Err(e)` prints `Error: {e:?}` (Debug, not Display)
//!   and exits with code 1.

use std::error::Error;
use std::thread;

type BoxError = Box<dyn Error + Send + Sync>;

/// Adds up the comma-separated numbers in `chunk` (spaces around them are
/// fine). A blank chunk is the error "empty chunk"; a bad number is the
/// `ParseIntError`.
fn sum_chunk(chunk: &str) -> Result<u64, BoxError> {
    if chunk.trim().is_empty() {
        return Err("empty chunk".into());
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
            total += worker.join().map_err(|_| "worker panicked")??;
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
