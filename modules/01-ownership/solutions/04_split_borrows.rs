//! # Splitting borrows (reference solution)
//!
//! - `self.record(..)` needs `&mut self` (all of it) while `player` still
//!   borrows `self.players`. Using the field directly, `self.log.push(..)`,
//!   borrows only `self.log`, which is disjoint. (Destructuring with
//!   `let Game { players, log } = self;` is the other classic way.)
//! - `slice::get_disjoint_mut` (Rust 1.86) hands out several `&mut`
//!   elements at once and checks the indices: out of bounds and overlapping
//!   indices come back as an error instead of a panic. Before 1.86 the
//!   answer was `split_at_mut`.
//! - Validate before mutating, so an error leaves no half-done change.

use std::slice::GetDisjointMutError;

#[derive(Debug, PartialEq)]
struct Player {
    name: String,
    score: u32,
}

#[derive(Debug, PartialEq)]
enum TransferError {
    NoSuchPlayer,
    SamePlayer,
    NotEnoughPoints,
}

struct Game {
    players: Vec<Player>,
    log: Vec<String>,
}

impl Game {
    /// Gives every player `points` and logs "<name> +<points>" for each.
    fn award_all(&mut self, points: u32) {
        for player in &mut self.players {
            player.score += points;
            self.log.push(format!("{} +{points}", player.name));
        }
    }

    /// Moves `amount` points from the player at position `from` to the one
    /// at `to`, logging "<from name> -> <to name>: <amount>". On error,
    /// nothing changes.
    fn transfer(&mut self, from: usize, to: usize, amount: u32) -> Result<(), TransferError> {
        let [giver, taker] = self
            .players
            .get_disjoint_mut([from, to])
            .map_err(|e| match e {
                GetDisjointMutError::IndexOutOfBounds => TransferError::NoSuchPlayer,
                GetDisjointMutError::OverlappingIndices => TransferError::SamePlayer,
            })?;
        if giver.score < amount {
            return Err(TransferError::NotEnoughPoints);
        }
        giver.score -= amount;
        taker.score += amount;
        self.log
            .push(format!("{} -> {}: {amount}", giver.name, taker.name));
        Ok(())
    }
}

fn new_game() -> Game {
    let player = |name: &str, score| Player {
        name: name.to_string(),
        score,
    };
    Game {
        players: vec![player("ann", 10), player("bob", 3), player("cy", 0)],
        log: Vec::new(),
    }
}

fn main() {
    let mut game = new_game();
    game.award_all(5);
    println!("{:?}", game.transfer(0, 2, 7));
    println!("{:?}", game.transfer(1, 1, 1));
    println!("{:?}", game.log);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scores(game: &Game) -> Vec<u32> {
        game.players.iter().map(|p| p.score).collect()
    }

    #[test]
    fn awards_and_logs() {
        let mut game = new_game();
        game.award_all(5);
        assert_eq!(scores(&game), [15, 8, 5]);
        assert_eq!(game.log, ["ann +5", "bob +5", "cy +5"]);
    }

    #[test]
    fn transfers_points() {
        let mut game = new_game();
        assert_eq!(game.transfer(0, 2, 7), Ok(()));
        assert_eq!(scores(&game), [3, 3, 7]);
        assert_eq!(game.log, ["ann -> cy: 7"]);
    }

    #[test]
    fn rejects_bad_transfers_without_changes() {
        let mut game = new_game();
        assert_eq!(game.transfer(1, 0, 4), Err(TransferError::NotEnoughPoints));
        assert_eq!(game.transfer(1, 1, 1), Err(TransferError::SamePlayer));
        assert_eq!(game.transfer(0, 9, 1), Err(TransferError::NoSuchPlayer));
        assert_eq!(scores(&game), [10, 3, 0]);
        assert!(game.log.is_empty());
    }
}
