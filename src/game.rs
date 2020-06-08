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
#[derive(Debug, PartialEq)]
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

/// The secret
#[derive(Debug, PartialEq)]
pub struct Secret {
    chars: Vec<HangmanChar>,
    pub chars_to_guess: usize,
}

impl Secret {
    /// Constructor.
    pub fn new(secretstr: &str) -> Self {
        // parse `secretsstr`, flip 'visible' every CONF_LINE_SECRET_MODIFIER__VISIBLE
        let w: Vec<HangmanChar> = secretstr
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

        let chars_to_guess = w.iter().filter(|hc| !hc.visible).count();

        Self {
            chars: w,
            chars_to_guess,
        }
    }

    pub fn visible_chars(&self) -> usize {
        self.chars.iter().filter(|hc| !hc.visible).count()
    }
}

impl fmt::Display for Secret {
    /// Graphical representation of the game state.
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut linebreak = false;
        for (n, c) in self.chars.iter().enumerate() {
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

/// The game state.
#[derive(Debug, PartialEq)]
pub struct Game {
    pub secret: Secret,
    pub lives: u8,
    pub last_guess: char,
    pub state: State,
    pub last_game: bool,
}

impl Game {
    /// Constructor.
    pub fn new(secretstr: &str, lives: u8, last_game: bool) -> Self {
        // parse `secretsstr`, flip 'visible' every CONF_LINE_SECRET_MODIFIER__VISIBLE
        let secret = Secret::new(secretstr);
        Self {
            secret,
            lives,
            last_guess: ' ',
            state: State::Ongoing,
            last_game,
        }
    }

    /// Process a guess and modify the game state.
    pub fn guess(&mut self, char_: char) {
        if char_ == '\n' {
            return;
        };
        self.last_guess = char_;
        let mut found = false;
        for h_char in &mut self.secret.chars {
            if h_char.char_.eq_ignore_ascii_case(&char_) {
                h_char.visible = true;
                found = true;
            }
        }

        if !found {
            self.lives -= 1;
        }

        self.state = if self.lives == 0 {
            // Disclose the secret
            for hc in &mut self.secret.chars {
                hc.visible = true;
            }
            if self.last_game {
                State::DefeatGameOver
            } else {
                State::Defeat
            }
        } else if self.secret.chars.iter().all(|c| c.visible) {
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
            self.lives, self.last_guess
        )?;

        write!(f, "{}", self.secret)
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
        let expected = Game {
            secret: Secret {
                chars: [
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
                chars_to_guess: 2,
            },
            lives: 2,
            last_guess: ' ',
            state: State::Ongoing,
            last_game: true,
        };

        assert_eq!(game, expected);

        // now we guess right
        game.guess('c');
        //println!("{:?}",game);
        let expected = Game {
            secret: Secret {
                chars: [
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
                chars_to_guess: 2,
            },
            lives: 2,
            last_guess: 'c',
            state: State::Ongoing,
            last_game: true,
        };

        assert_eq!(game, expected);

        // now we guess wrong
        game.guess('x');
        //println!("{:?}",game);
        let expected = Game {
            secret: Secret {
                chars: [
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
                chars_to_guess: 2,
            },
            lives: 1,
            last_guess: 'x',
            state: State::Ongoing,
            last_game: true,
        };

        assert_eq!(game, expected);

        // we guess wrong again and we loose
        game.guess('y');
        //println!("{:?}",game);
        let expected = Game {
            secret: Secret {
                chars: [
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
                chars_to_guess: 2,
            },
            lives: 0,
            last_guess: 'y',
            state: State::DefeatGameOver,
            last_game: true,
        };

        assert_eq!(game, expected);
    }
}
