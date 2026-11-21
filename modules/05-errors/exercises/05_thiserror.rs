//! # Deriving errors with thiserror
//!
//! `#[derive(thiserror::Error)]` writes `Display`, `Error` and `From` for
//! you. `#[error("..")]` is the message (fields interpolate: `{name}`,
//! `{0}`, `{text:?}`); a field named `source`, or marked `#[source]`, is
//! the cause; `#[from]` also generates a `From` impl; and
//! `#[error(transparent)]` forwards message and cause to the one wrapped
//! error. This backup tool's error type doesn't compile. Fix it, and the
//! code that relies on it, keeping the messages the docs give.

use std::io::{self, Read, Write};
use std::num::ParseIntError;

use thiserror::Error;

#[derive(Debug, Error)]
enum ManifestError {
    #[error("line {line}: expected \"<name> <size>\"")]
    Malformed { line: usize },
    #[error("line {line}: bad size {text:?}")]
    BadSize {
        line: usize,
        text: String,
        source: ParseIntError,
    },
    #[error("no entry for {0}")]
    Missing(String),
}

#[derive(Debug, Error)]
enum BackupError {
    /// Reading the source failed: "reading <name>".
    #[error("reading {name}")]
    Read {
        name: String,
        #[from]
        source: io::Error,
    },
    /// Writing the copy failed: "writing <name>".
    #[error("writing {name}")]
    Write {
        name: String,
        #[from]
        source: io::Error,
    },
    /// The copy doesn't match the manifest.
    #[error("{name}: the manifest says {expected} bytes, copied {copied}")]
    SizeMismatch {
        name: String,
        expected: u64,
        copied: u64,
    },
    /// The manifest is broken. Message and cause are the `ManifestError`'s.
    #[error(transparent)]
    Manifest(ManifestError),
}

/// The size the manifest lists for `name`. Each manifest line is
/// "<name> <size>".
fn manifest_size(manifest: &str, name: &str) -> Result<u64, ManifestError> {
    for (i, line) in manifest.lines().enumerate() {
        let (entry, size) = line
            .split_once(' ')
            .ok_or(ManifestError::Malformed { line: i + 1 })?;
        if entry == name {
            return size.parse().map_err(|source| ManifestError::BadSize {
                line: i + 1,
                text: size.to_string(),
                source,
            });
        }
    }
    Err(ManifestError::Missing(name.to_string()))
}

/// Copies everything from `src` to `dst` and returns the number of bytes.
/// `name` is the file's name, for error messages.
fn back_up(name: &str, mut src: impl Read, mut dst: impl Write) -> Result<u64, BackupError> {
    let mut data = Vec::new();
    src.read_to_end(&mut data)?;
    dst.write_all(&data)?;
    Ok(data.len() as u64)
}

/// Like `back_up`, but first looks up the size of `name` in the manifest,
/// and afterwards checks that the right number of bytes was copied.
fn back_up_checked(
    manifest: &str,
    name: &str,
    src: impl Read,
    dst: impl Write,
) -> Result<u64, BackupError> {
    let expected = manifest_size(manifest, name)?;
    let copied = back_up(name, src, dst)?;
    if copied != expected {
        return Err(BackupError::SizeMismatch {
            name: name.to_string(),
            expected,
            copied,
        });
    }
    Ok(copied)
}

fn main() {
    let manifest = "notes.txt 5\ntodo.txt 3";
    let mut copy = Vec::new();
    println!(
        "{:?}",
        back_up_checked(manifest, "notes.txt", &b"hello"[..], &mut copy)
    );
    for name in ["todo.txt", "photo.jpg"] {
        if let Err(e) = back_up_checked(manifest, name, &b"hello"[..], &mut copy) {
            println!("{e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    /// A reader whose disk has gone away.
    struct Unplugged;

    impl Read for Unplugged {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("disk unplugged"))
        }
    }

    /// A writer whose disk is full.
    struct Full;

    impl Write for Full {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("no space left"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn chain(error: &dyn Error) -> Vec<String> {
        let mut messages = vec![error.to_string()];
        let mut current = error.source();
        while let Some(cause) = current {
            messages.push(cause.to_string());
            current = cause.source();
        }
        messages
    }

    #[test]
    fn copies() {
        let mut copy = Vec::new();
        let copied = back_up("notes.txt", &b"hello"[..], &mut copy).unwrap();
        assert_eq!(copied, 5);
        assert_eq!(copy, b"hello");
    }

    #[test]
    fn read_and_write_errors_say_which_and_what() {
        let err = back_up("notes.txt", Unplugged, Vec::new()).unwrap_err();
        assert!(matches!(err, BackupError::Read { .. }));
        assert_eq!(chain(&err), ["reading notes.txt", "disk unplugged"]);

        let err = back_up("notes.txt", &b"hello"[..], Full).unwrap_err();
        assert!(matches!(err, BackupError::Write { .. }));
        assert_eq!(chain(&err), ["writing notes.txt", "no space left"]);
    }

    #[test]
    fn checks_the_size() {
        let manifest = "notes.txt 5\ntodo.txt 3";
        let ok = back_up_checked(manifest, "notes.txt", &b"hello"[..], Vec::new());
        assert_eq!(ok.unwrap(), 5);
        let err = back_up_checked(manifest, "todo.txt", &b"hello"[..], Vec::new()).unwrap_err();
        assert_eq!(
            chain(&err),
            ["todo.txt: the manifest says 3 bytes, copied 5"]
        );
    }

    #[test]
    fn manifest_errors_are_transparent() {
        let err = back_up_checked("a.txt 1", "b.txt", Unplugged, Vec::new()).unwrap_err();
        assert!(matches!(
            err,
            BackupError::Manifest(ManifestError::Missing(_))
        ));
        assert_eq!(chain(&err), ["no entry for b.txt"]);

        let manifest = "a.txt 1\nb.txt lots";
        let err = back_up_checked(manifest, "b.txt", Unplugged, Vec::new()).unwrap_err();
        assert_eq!(
            chain(&err),
            ["line 2: bad size \"lots\"", "invalid digit found in string"]
        );
    }

    #[test]
    fn manifest_errors_convert_with_from() {
        let err = BackupError::from(ManifestError::Malformed { line: 4 });
        assert_eq!(err.to_string(), "line 4: expected \"<name> <size>\"");
    }
}
