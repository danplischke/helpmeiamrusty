//! # Adaptor fluency
//!
//! Five questions about a bank statement. Each answer is one iterator
//! chain, with no `for` and no `let mut`: pick the adaptor that says what
//! you mean (`filter_map`, `scan`, `take_while`, `flat_map`, `zip`,
//! `chain`, `rfind`) instead of a loop with flags and counters.
//!
//! Amounts are in cents: deposits are positive, withdrawals negative.

/// One amount per line. Lines that aren't a whole number once trimmed
/// (headers, blank lines, "n/a", "12.50") are skipped.
fn parse_amounts(export: &str) -> Vec<i64> {
    todo!("parse {export:?}")
}

/// The balance after each transaction, starting from `opening`. Stops for
/// good before the first transaction that would take the balance below
/// zero, even if a later deposit would bring it back up.
fn balances_until_overdrawn(opening: i64, amounts: &[i64]) -> Vec<i64> {
    todo!("balances from {opening} through {amounts:?}")
}

/// Every `#tag` in the memos, in order, lowercased and without the `#`.
/// Tags are whitespace-separated words that start with `#`; a lone `#` is
/// not a tag, and neither is `mid#word`.
fn tags(memos: &[&str]) -> Vec<String> {
    todo!("tags in {memos:?}")
}

/// The positions where our records and the bank's disagree. If one list is
/// longer, each of its extra positions counts as a disagreement too.
fn mismatches(ours: &[i64], bank: &[i64]) -> Vec<usize> {
    todo!("compare {ours:?} with {bank:?}")
}

/// The most recent (last) withdrawal, if there is one.
fn last_withdrawal(amounts: &[i64]) -> Option<i64> {
    todo!("last withdrawal in {amounts:?}")
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
