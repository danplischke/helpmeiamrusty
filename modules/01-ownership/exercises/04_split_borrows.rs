//! # Splitting borrows
//!
//! Inside one function body the borrow checker tracks struct *fields*
//! separately, but a method call like `self.record(..)` borrows all of
//! `self`. It also can't tell that `v[i]` and `v[j]` are different elements.
//! Fix both problems.
//!
//! Rules: no `.clone()`, no `[indexing]` (use methods that return `Option`
//! or `Result`), no `unsafe`.

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
    fn record(&mut self, message: String) {
        self.log.push(message);
    }

    /// Gives every player `points` and logs "<name> +<points>" for each.
    fn award_all(&mut self, points: u32) {
        for player in self.players.iter_mut() {
            player.score += points;
            self.record(format!("{} +{points}", player.name));
        }
    }

    /// Moves `amount` points from the player at position `from` to the one
    /// at `to`, logging "<from name> -> <to name>: <amount>". On error,
    /// nothing changes.
    fn transfer(&mut self, from: usize, to: usize, amount: u32) -> Result<(), TransferError> {
        let giver = &mut self.players[from];
        let taker = &mut self.players[to];
        if giver.score < amount {
            return Err(TransferError::NotEnoughPoints);
        }
        giver.score -= amount;
        taker.score += amount;
        self.record(format!("{} -> {}: {amount}", giver.name, taker.name));
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
