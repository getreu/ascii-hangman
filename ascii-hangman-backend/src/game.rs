//!Defines the game state and logic
use crate::secret::Secret;
use std::fmt;

/// A subset of the game state. Can be derived from `Game` struct.
#[derive(Debug, PartialEq, Clone)]
pub enum State {
    /// The game is ongoing.
    Ongoing,
    /// The player won and the game is continuable.
    Victory,
    /// The player lost and the game is continuable.
    Defeat,
    /// The player won and there are no more words to guess.
    VictoryGameOver,
    /// The player lost and there are no more secrets to guess.
    DefeatGameOver,
}

/// The game state.
#[derive(Debug, PartialEq)]
pub struct Game {
    pub secret: Secret,
    pub lifes: u8,
    pub last_guess: char,
    pub state: State,
    pub last_game: bool,
}

impl Game {
    /// Constructor.
    pub fn new(secretstr: &str, lifes: u8, last_game: bool) -> Self {
        // parse `secretsstr`, flip 'visible' every CONF_LINE_SECRET_MODIFIER__VISIBLE
        let secret = Secret::new(secretstr);
        Self {
            secret,
            lifes,
            last_guess: ' ',
            state: State::Ongoing,
            last_game,
        }
    }

    /// Process a guess and modify the game state.
    pub fn guess(&mut self, character: char) {
        if character == '\n' {
            return;
        };
        self.last_guess = character;

        let found = self.secret.guess(character);

        if !found {
            self.lifes -= 1;
        }

        self.state = if self.lifes == 0 {
            // Disclose the secret
            self.secret.disclose_all();

            if self.last_game {
                State::DefeatGameOver
            } else {
                State::Defeat
            }
        } else if self.secret.is_fully_disclosed() {
            if self.last_game {
                State::VictoryGameOver
            } else {
                State::Victory
            }
        } else {
            State::Ongoing
        };
    }
}

impl fmt::Display for Game {
    /// Graphical representation of the game state.
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        writeln!(
            f,
            "Lives:\t{}\tLast guess: {}\n",
            self.lifes, self.last_guess
        )
    }
}
// ***********************

#[cfg(test)]
mod tests {
    use super::*;

    /// Play simulation
    #[test]
    fn test_game_simulation() {
        let mut game = Game::new("_ab _cd", 2, true);
        //println!("{:?}",game);

        assert_eq!(format!("{}", game.secret), " a b   _ _\n");
        assert_eq!(game.lifes, 2);
        assert_eq!(game.last_guess, ' ');
        assert_eq!(game.state, State::Ongoing);
        assert_eq!(game.last_game, true);

        // now we guess right
        game.guess('c');
        //println!("{:?}",game);

        assert_eq!(format!("{}", game.secret), " a b   c _\n");
        assert_eq!(game.lifes, 2);
        assert_eq!(game.last_guess, 'c');
        assert_eq!(game.state, State::Ongoing);
        assert_eq!(game.last_game, true);

        // now we guess wrong
        game.guess('x');
        //println!("{:?}",game);

        assert_eq!(format!("{}", game.secret), " a b   c _\n");
        assert_eq!(game.lifes, 1);
        assert_eq!(game.last_guess, 'x');
        assert_eq!(game.state, State::Ongoing);
        assert_eq!(game.last_game, true);

        // we guess wrong again and we loose
        game.guess('y');
        //println!("{:?}",game);
        assert_eq!(format!("{}", game.secret), " a b   c d\n");
        assert_eq!(game.lifes, 0);
        assert_eq!(game.last_guess, 'y');
        assert_eq!(game.state, State::DefeatGameOver);
        assert_eq!(game.last_game, true);
    }
}
