//! # Adaptor fluency (reference solution)
//!
//! - `filter_map` is "keep some, transforming them" in one step;
//!   `.parse().ok()` turns a failed parse into a skipped line.
//! - `scan` threads state through an iterator and yields as it goes;
//!   `take_while` then stops at the first item that fails. `filter` would
//!   skip the overdrawn balances and carry on, which is a different answer.
//! - `flat_map` turns each memo into many words; `strip_prefix` returns an
//!   `Option`, so it slots straight into `filter_map`.
//! - `zip` stops at the shorter input without telling you. When the extra
//!   items matter, `chain` them on yourself.
//! - `rfind` searches from the back. `.filter(..).last()` gives the same
//!   answer but walks everything first (clippy: `double_ended_iterator_last`).

/// One amount per line. Lines that aren't a whole number once trimmed
/// (headers, blank lines, "n/a", "12.50") are skipped.
fn parse_amounts(export: &str) -> Vec<i64> {
    export
        .lines()
        .filter_map(|line| line.trim().parse().ok())
        .collect()
}

/// The balance after each transaction, starting from `opening`. Stops for
/// good before the first transaction that would take the balance below
/// zero, even if a later deposit would bring it back up.
fn balances_until_overdrawn(opening: i64, amounts: &[i64]) -> Vec<i64> {
    amounts
        .iter()
        .scan(opening, |balance, &amount| {
            *balance += amount;
            Some(*balance)
        })
        .take_while(|&balance| balance >= 0)
        .collect()
}

/// Every `#tag` in the memos, in order, lowercased and without the `#`.
/// Tags are whitespace-separated words that start with `#`; a lone `#` is
/// not a tag, and neither is `mid#word`.
fn tags(memos: &[&str]) -> Vec<String> {
    memos
        .iter()
        .flat_map(|memo| memo.split_whitespace())
        .filter_map(|word| word.strip_prefix('#'))
        .filter(|tag| !tag.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// The positions where our records and the bank's disagree. If one list is
/// longer, each of its extra positions counts as a disagreement too.
fn mismatches(ours: &[i64], bank: &[i64]) -> Vec<usize> {
    let shorter = ours.len().min(bank.len());
    let longer = ours.len().max(bank.len());
    ours.iter()
        .zip(bank)
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(i, _)| i)
        .chain(shorter..longer)
        .collect()
}

/// The most recent (last) withdrawal, if there is one.
fn last_withdrawal(amounts: &[i64]) -> Option<i64> {
    amounts.iter().copied().rfind(|&amount| amount < 0)
}

fn main() {
    let amounts = parse_amounts("amount\n2500\n-1200\nn/a\n-1800\n400\n");
    println!("amounts: {amounts:?}");
    println!("balances: {:?}", balances_until_overdrawn(0, &amounts));
    println!("tags: {:?}", tags(&["#Rent for May", "coffee #treat"]));
    println!(
        "mismatches: {:?}",
        mismatches(&amounts, &[2500, -1200, -1700])
    );
    println!("last withdrawal: {:?}", last_withdrawal(&amounts));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_amounts_skipping_noise() {
        let export = "amount\n1250\n\n  -300  \nn/a\n12.50\n75\n";
        assert_eq!(parse_amounts(export), [1250, -300, 75]);
        assert!(parse_amounts("").is_empty());
    }

    #[test]
    fn balances_stop_before_an_overdraft() {
        assert_eq!(
            balances_until_overdrawn(100, &[50, -120, -40, 30]),
            [150, 30]
        );
        assert_eq!(balances_until_overdrawn(0, &[0, 5, -5]), [0, 5, 0]);
        assert!(balances_until_overdrawn(10, &[]).is_empty());
    }

    #[test]
    fn balances_stop_for_good() {
        assert!(
            balances_until_overdrawn(100, &[-150, 200]).is_empty(),
            "a later deposit doesn't restart the list"
        );
        assert_eq!(balances_until_overdrawn(10, &[5, -20, 100, 1]), [15]);
    }

    #[test]
    fn collects_tags() {
        assert_eq!(
            tags(&["#Rent for May", "coffee #treat #FOOD"]),
            ["rent", "treat", "food"]
        );
        assert!(tags(&["no tags here", "# alone", "mid#word"]).is_empty());
        assert!(tags(&[]).is_empty());
    }

    #[test]
    fn finds_mismatches() {
        assert_eq!(mismatches(&[10, 20, 30], &[10, 25, 30]), [1]);
        assert!(mismatches(&[10, 20], &[10, 20]).is_empty());
        assert!(mismatches(&[], &[]).is_empty());
    }

    #[test]
    fn extra_entries_are_mismatches() {
        assert_eq!(mismatches(&[10, 20], &[10, 20, 5, 7]), [2, 3]);
        assert_eq!(mismatches(&[1, 2, 3], &[9]), [0, 1, 2]);
    }

    #[test]
    fn finds_the_last_withdrawal() {
        assert_eq!(last_withdrawal(&[100, -20, 50, -5, 10]), Some(-5));
        assert_eq!(last_withdrawal(&[100, 0]), None);
        assert_eq!(last_withdrawal(&[]), None);
    }
}
