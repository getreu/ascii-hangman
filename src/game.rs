//!Defines the game state and logic

use crate::dictionary::CONF_LINE_SECRET_MODIFIER__VISIBLE;
use std::fmt;

/// Defines the line-break position when displaying the secret string.
const LINE_WIDTH: usize = 20;

/// One character of the secret string.
#[derive(Debug, Clone, PartialEq)]
struct HangmanChar {
    char_: char,
    visible: bool,
}

/// Format HangmanChar.
impl fmt::Display for HangmanChar {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if self.visible {
            write!(f, "{}", self.char_)
        } else {
            write!(f, "_")
        }
    }
}

/// A subset of the game state. Can be derived from `Game` struct.
#[derive(PartialEq)]
pub enum State {
    /// The game is ongoing.
    Ongoing,
    /// The player won.
    Victory,
    /// The player lost.
    Defeat,
}

/// The game state.
#[derive(Debug, PartialEq)]
pub struct Game {
    secret: Vec<HangmanChar>,
    pub lives: u8,
    pub last_guess: char,
}

impl Game {
    /// Derive State from Game data.
    pub fn get_state(&self) -> State {
        if self.lives == 0 {
            State::Defeat
        } else if self.secret.iter().all(|c| c.visible) {
            State::Victory
        } else {
            State::Ongoing
        }
    }

    /// Constructor.
    pub fn new(secretstr: &str, lives: u8) -> Self {
        // parse `secretsstr`, flip 'visible' every CONF_LINE_SECRET_MODIFIER__VISIBLE
        let w = secretstr
            .chars()
            // for every * found flip v_acc
            .scan(false, |v_acc, c| {
                *v_acc ^= c == CONF_LINE_SECRET_MODIFIER__VISIBLE;
                if c == CONF_LINE_SECRET_MODIFIER__VISIBLE {
                    Some(None)
                } else {
                    Some(Some(HangmanChar {
                        char_: c,
                        visible: *v_acc,
                    }))
                }
            })
            // omit None and unwrap
            .filter_map(|s| s)
            //.inspect(|ref x| println!("after scan:\t{:?}", x))
            .collect();

        Self {
            secret: w,
            lives,
            last_guess: ' ',
        }
    }

    /// Process a guess and modify the game state.
    pub fn guess(&mut self, char_: char) {
        if char_ == '\n' {
            return;
        };
        self.last_guess = char_;
        let mut found = false;
        for h_char in &mut self.secret {
            if h_char.char_.eq_ignore_ascii_case(&char_) {
                h_char.visible = true;
                found = true;
            }
        }

        if !found {
            self.lives -= 1;
        }

        if self.lives == 0 {
            for hc in &mut self.secret {
                hc.visible = true;
            }
        }
    }

    /// The number of disclosed characters of the secret.
    pub fn visible_chars(&self) -> usize {
        self.secret.iter().filter(|hc| !hc.visible).count()
    }
}

impl fmt::Display for Game {
    /// Graphical representation of the game state.
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        writeln!(
            f,
            "Lives:\t{}\tLast guess: {}\n",
            self.lives, self.last_guess
        )?;

        let mut linebreak = false;
        for (n, c) in self.secret.iter().enumerate() {
            if n % LINE_WIDTH == 0 {
                linebreak = true
            };
            if n == 0 {
                linebreak = false
            };
            if linebreak && (c.char_ == ' ') {
                linebreak = false;
                writeln!(f, " {}", c)?
            } else {
                write!(f, " {}", c)?
            }
        }
        writeln!(f)
    }
}
// ***********************

#[cfg(test)]
mod tests {
    use super::*;

    /// Play simulation
    #[test]
    fn test_game_simulation() {
        let mut game = Game::new("_ab _cd", 2);

        //println!("{:?}",game);
        let expected = Game {
            secret: [
                HangmanChar {
                    char_: 'a',
                    visible: true,
                },
                HangmanChar {
                    char_: 'b',
                    visible: true,
                },
                HangmanChar {
                    char_: ' ',
                    visible: true,
                },
                HangmanChar {
                    char_: 'c',
                    visible: false,
                },
                HangmanChar {
                    char_: 'd',
                    visible: false,
                },
            ]
            .to_vec(),
            lives: 2,
            last_guess: ' ',
        };

        assert!(game == expected);
        assert!(game.get_state() == State::Ongoing);

        // now we guess right
        game.guess('c');
        //println!("{:?}",game);
        let expected = Game {
            secret: [
                HangmanChar {
                    char_: 'a',
                    visible: true,
                },
                HangmanChar {
                    char_: 'b',
                    visible: true,
                },
                HangmanChar {
                    char_: ' ',
                    visible: true,
                },
                HangmanChar {
                    char_: 'c',
                    visible: true,
                },
                HangmanChar {
                    char_: 'd',
                    visible: false,
                },
            ]
            .to_vec(),
            lives: 2,
            last_guess: 'c',
        };

        assert!(game == expected);
        assert!(game.get_state() == State::Ongoing);

        // now we guess wrong
        game.guess('x');
        //println!("{:?}",game);
        let expected = Game {
            secret: [
                HangmanChar {
                    char_: 'a',
                    visible: true,
                },
                HangmanChar {
                    char_: 'b',
                    visible: true,
                },
                HangmanChar {
                    char_: ' ',
                    visible: true,
                },
                HangmanChar {
                    char_: 'c',
                    visible: true,
                },
                HangmanChar {
                    char_: 'd',
                    visible: false,
                },
            ]
            .to_vec(),
            lives: 1,
            last_guess: 'x',
        };

        assert!(game == expected);
        assert!(game.get_state() == State::Ongoing);

        // we guess wrong again and we loose
        game.guess('y');
        //println!("{:?}",game);
        let expected = Game {
            secret: [
                HangmanChar {
                    char_: 'a',
                    visible: true,
                },
                HangmanChar {
                    char_: 'b',
                    visible: true,
                },
                HangmanChar {
                    char_: ' ',
                    visible: true,
                },
                HangmanChar {
                    char_: 'c',
                    visible: true,
                },
                HangmanChar {
                    char_: 'd',
                    visible: true,
                },
            ]
            .to_vec(),
            lives: 0,
            last_guess: 'y',
        };

        assert!(game == expected);
        assert!(game.get_state() == State::Defeat);
    }
}
