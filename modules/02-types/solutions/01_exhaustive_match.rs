//! # Exhaustive matching as a refactoring checklist (reference solution)
//!
//! - E0004 is a feature: add a variant and the compiler lists every `match`
//!   that has to decide what it means. A `_` arm opts out of that list,
//!   which is how `can_cover` silently let gift cards pay any amount.
//! - On enums you own, prefer naming the variants (`Cash | BankTransfer { .. }`)
//!   to `_`. Keep `_` for types with too many values to list (integers,
//!   strings) or for enums from other crates marked `#[non_exhaustive]`.
//! - An or-pattern may bind the same name in each alternative, even from
//!   differently named fields: `Card { credit_left: limit, .. } | GiftCard
//!   { balance: limit, .. }`. The types have to agree.
//! - Clippy can enforce it: `wildcard_enum_match_arm` (restriction) flags
//!   every `_` arm on an enum, and `match_wildcard_for_single_variants`
//!   (pedantic) one that stands for a single variant.

#[derive(Debug, PartialEq)]
enum PaymentMethod {
    Cash,
    Card {
        last4: String,
        credit_left: u64,
    },
    BankTransfer {
        iban: String,
    },
    /// New: a prepaid card. `balance` is in cents, like every amount here.
    GiftCard {
        code: String,
        balance: u64,
    },
}

/// The processing fee for a payment of `amount` cents: cash and gift cards
/// are free, cards cost 2% (rounded down), bank transfers a flat 30 cents.
fn fee(method: &PaymentMethod, amount: u64) -> u64 {
    match method {
        PaymentMethod::Cash => 0,
        PaymentMethod::Card { .. } => amount / 50,
        PaymentMethod::BankTransfer { .. } => 30,
        PaymentMethod::GiftCard { .. } => 0,
    }
}

/// How the method appears on a receipt: "cash", "card ending 4242",
/// "transfer from DE89370400440532013000", "gift card GC-7781".
fn receipt_label(method: &PaymentMethod) -> String {
    match method {
        PaymentMethod::Cash => "cash".to_string(),
        PaymentMethod::Card { last4, .. } => format!("card ending {last4}"),
        PaymentMethod::BankTransfer { iban } => format!("transfer from {iban}"),
        PaymentMethod::GiftCard { code, .. } => format!("gift card {code}"),
    }
}

/// Whether the method can pay `amount` cents. Cash and transfers always
/// can. A card can pay up to its remaining credit, a gift card up to its
/// balance.
fn can_cover(method: &PaymentMethod, amount: u64) -> bool {
    match method {
        PaymentMethod::Cash | PaymentMethod::BankTransfer { .. } => true,
        PaymentMethod::Card {
            credit_left: limit, ..
        }
        | PaymentMethod::GiftCard { balance: limit, .. } => amount <= *limit,
    }
}

fn main() {
    let gift = PaymentMethod::GiftCard {
        code: "GC-7781".to_string(),
        balance: 2000,
    };
    println!("{}", receipt_label(&gift));
    println!("fee on 12.00: {} cents", fee(&gift, 1200));
    println!("covers 25.00? {}", can_cover(&gift, 2500));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn methods() -> [PaymentMethod; 4] {
        [
            PaymentMethod::Cash,
            PaymentMethod::Card {
                last4: "4242".to_string(),
                credit_left: 5000,
            },
            PaymentMethod::BankTransfer {
                iban: "DE89370400440532013000".to_string(),
            },
            PaymentMethod::GiftCard {
                code: "GC-7781".to_string(),
                balance: 2000,
            },
        ]
    }

    #[test]
    fn fees() {
        let [cash, card, transfer, gift] = methods();
        assert_eq!(fee(&cash, 1000), 0);
        assert_eq!(fee(&card, 1000), 20);
        assert_eq!(fee(&card, 49), 0);
        assert_eq!(fee(&transfer, 1000), 30);
        assert_eq!(fee(&gift, 1000), 0);
    }

    #[test]
    fn receipt_labels() {
        let labels = methods().map(|method| receipt_label(&method));
        assert_eq!(
            labels,
            [
                "cash",
                "card ending 4242",
                "transfer from DE89370400440532013000",
                "gift card GC-7781",
            ]
        );
    }

    #[test]
    fn coverage() {
        let [cash, card, transfer, gift] = methods();
        assert!(can_cover(&cash, 1_000_000));
        assert!(can_cover(&transfer, 1_000_000));
        assert!(can_cover(&card, 5000));
        assert!(!can_cover(&card, 5001));
        assert!(can_cover(&gift, 2000));
        assert!(
            !can_cover(&gift, 2001),
            "a gift card can't pay more than its balance"
        );
    }
}
