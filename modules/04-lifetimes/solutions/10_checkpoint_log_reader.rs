//! # Checkpoint: a zero-copy log reader (reference solution)
//!
//! - Every field of `Record<'a>` and `BadLine<'a>` is a slice of the input
//!   line. `splitn`, `split_once` and `trim` all return slices of what you
//!   give them, so no `String` appears anywhere.
//! - `field` reads `self.message`, a `&'a str`, so what it finds is `'a`
//!   too, and outlives the record. `count_by_target` returns `&'a str`
//!   targets for the same reason, while `at_least` hands out `&'r Record`s:
//!   two lifetimes, because the result borrows the slice *and* the text.
//! - `Iterator for Reader<'a>` can yield `Record<'a>` because the records
//!   borrow the text, not the reader.
//! - `at_least` returns a lazy iterator. Edition 2024 captures `'r` and
//!   `'a` without being told; the closure needs `move` so it owns its copy
//!   of `level` instead of borrowing a local that's about to die (E0373).

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

impl Level {
    /// "DEBUG", "INFO", "WARN" or "ERROR".
    fn parse(s: &str) -> Option<Level> {
        match s {
            "DEBUG" => Some(Level::Debug),
            "INFO" => Some(Level::Info),
            "WARN" => Some(Level::Warn),
            "ERROR" => Some(Level::Error),
            _ => None,
        }
    }
}

/// One log line: "<time> <LEVEL> <target>: <message>".
#[derive(Debug, PartialEq)]
struct Record<'a> {
    line_no: usize,
    time: &'a str,
    level: Level,
    target: &'a str,
    message: &'a str,
}

/// A line that doesn't parse, kept as it is.
#[derive(Debug, PartialEq)]
struct BadLine<'a> {
    line_no: usize,
    line: &'a str,
}

impl<'a> Record<'a> {
    /// Parses one line; `line_no` is stored as given. The line splits at its
    /// first two spaces into time, level and the rest. The target is the
    /// rest up to its first ':', and must be non-empty with no spaces. The
    /// message is what follows that ':', trimmed; it may be empty. Any other
    /// line, or an unknown level, gives a `BadLine` with the whole line.
    fn parse(line_no: usize, line: &'a str) -> Result<Record<'a>, BadLine<'a>> {
        let mut parts = line.splitn(3, ' ');
        let (Some(time), Some(level), Some(rest)) = (parts.next(), parts.next(), parts.next())
        else {
            return Err(BadLine { line_no, line });
        };
        let (Some(level), Some((target, message))) = (Level::parse(level), rest.split_once(':'))
        else {
            return Err(BadLine { line_no, line });
        };
        if target.is_empty() || target.contains(' ') {
            return Err(BadLine { line_no, line });
        }
        Ok(Record {
            line_no,
            time,
            level,
            target,
            message: message.trim(),
        })
    }

    /// The value of `key` in the message's whitespace-separated `key=value`
    /// words: in "took=1200ms table=users", `field("table")` is "users".
    /// Words without '=' are ignored. The first match wins.
    fn field(&self, key: &str) -> Option<&'a str> {
        self.message
            .split_whitespace()
            .filter_map(|word| word.split_once('='))
            .find(|&(name, _)| name == key)
            .map(|(_, value)| value)
    }
}

/// Reads records from a log text. Blank lines (empty or only whitespace)
/// are skipped, but still counted: line numbers start at 1.
struct Reader<'a> {
    lines: std::iter::Enumerate<std::str::Lines<'a>>,
}

fn read(text: &str) -> Reader<'_> {
    Reader {
        lines: text.lines().enumerate(),
    }
}

impl<'a> Iterator for Reader<'a> {
    type Item = Result<Record<'a>, BadLine<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let (index, line) = self.lines.find(|(_, line)| !line.trim().is_empty())?;
        Some(Record::parse(index + 1, line))
    }
}

/// Each target with its number of records, in order of first appearance.
fn count_by_target<'a>(records: &[Record<'a>]) -> Vec<(&'a str, usize)> {
    let mut counts: Vec<(&'a str, usize)> = Vec::new();
    for record in records {
        match counts
            .iter_mut()
            .find(|(target, _)| *target == record.target)
        {
            Some((_, count)) => *count += 1,
            None => counts.push((record.target, 1)),
        }
    }
    counts
}

/// The records at `level` or above, lazily, in their original order.
fn at_least<'r, 'a>(
    records: &'r [Record<'a>],
    level: Level,
) -> impl Iterator<Item = &'r Record<'a>> {
    records.iter().filter(move |record| record.level >= level)
}

const LOG: &str = "\
12:00:01 INFO http: started port=8080
12:00:02 WARN db: slow query took=1200ms table=users

12:00:03 ERROR db: connection lost retry=3
garbage line
12:00:04 INFO http: request path=/users status=200
12:00:05 LOUD http: unknown level";

fn main() {
    let mut records = Vec::new();
    for result in read(LOG) {
        match result {
            Ok(record) => records.push(record),
            Err(bad) => println!("line {}: can't parse {:?}", bad.line_no, bad.line),
        }
    }
    println!("{:?}", count_by_target(&records));
    for record in at_least(&records, Level::Warn) {
        println!("{} {:?} {}", record.time, record.level, record.message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn records(text: &str) -> Vec<Record<'_>> {
        read(text).filter_map(Result::ok).collect()
    }

    #[test]
    fn parses_a_line() {
        assert_eq!(
            Record::parse(9, "12:00:02 WARN db: slow query took=1200ms"),
            Ok(Record {
                line_no: 9,
                time: "12:00:02",
                level: Level::Warn,
                target: "db",
                message: "slow query took=1200ms",
            })
        );
        let quiet = Record::parse(1, "t DEBUG app:").map(|r| r.message);
        assert_eq!(quiet, Ok(""));
    }

    #[test]
    fn rejects_bad_lines() {
        for line in [
            "garbage line",
            "12:00 LOUD http: x",
            "12:00 INFO http no colon",
            "12:00 INFO : no target",
            "12:00 INFO two words: no",
            "12:00 INFO",
        ] {
            assert_eq!(Record::parse(3, line), Err(BadLine { line_no: 3, line }));
        }
    }

    #[test]
    fn reads_and_numbers_lines() {
        let results: Vec<_> = read(LOG).collect();
        assert_eq!(results.len(), 6);
        let numbers: Vec<usize> = results
            .iter()
            .map(|result| match result {
                Ok(record) => record.line_no,
                Err(bad) => bad.line_no,
            })
            .collect();
        assert_eq!(numbers, [1, 2, 4, 5, 6, 7]);
        assert_eq!(
            results[3],
            Err(BadLine {
                line_no: 5,
                line: "garbage line",
            })
        );
        assert_eq!(read("").count(), 0);
        assert_eq!(read(" \n\t\n").count(), 0);
    }

    #[test]
    fn fields() {
        let all = records(LOG);
        assert_eq!(all[1].field("took"), Some("1200ms"));
        assert_eq!(all[1].field("table"), Some("users"));
        assert_eq!(all[1].field("slow"), None);
        assert_eq!(all[0].field("status"), None);
        let record =
            Record::parse(1, "t INFO x: a=1 a=2 b=c=d").map(|r| (r.field("a"), r.field("b")));
        assert_eq!(record, Ok((Some("1"), Some("c=d"))));
    }

    #[test]
    fn results_outlive_the_records() {
        let text = String::from(LOG);
        let (targets, table) = {
            let all = records(&text);
            (count_by_target(&all), all[1].field("table"))
        }; // the records are gone; what we kept borrows from `text`
        assert_eq!(targets, [("http", 2), ("db", 2)]);
        assert_eq!(table, Some("users"));
        assert!(count_by_target(&[]).is_empty());
    }

    #[test]
    fn records_point_into_the_text() {
        let text = String::from(LOG);
        let start = text.as_ptr() as usize;
        let inside = start..start + text.len();
        for record in records(&text) {
            for part in [record.time, record.target, record.message] {
                assert!(
                    inside.contains(&(part.as_ptr() as usize)),
                    "{part:?} was copied"
                );
            }
        }
    }

    #[test]
    fn filters_by_level() {
        let all = records(LOG);
        let serious: Vec<usize> = at_least(&all, Level::Warn).map(|r| r.line_no).collect();
        assert_eq!(serious, [2, 4]);
        assert_eq!(at_least(&all, Level::Debug).count(), 4);
        assert_eq!(at_least(&all[..1], Level::Error).next(), None);
    }
}
