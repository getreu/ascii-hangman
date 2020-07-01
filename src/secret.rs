use crate::dictionary::CONF_LINE_SECRET_MODIFIER__VISIBLE;
use std::fmt;

/// Defines the line-break position when displaying the secret string.
const LINE_WIDTH: usize = 20;

/// One character of the secret string.
#[derive(Clone, Debug, PartialEq)]
struct HangmanChar {
    character: char,
    visible: bool,
}

/// Format HangmanChar.
impl fmt::Display for HangmanChar {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if self.visible {
            write!(f, "{}", self.character)
        } else {
            write!(f, "_")
        }
    }
}

/// The secret
#[derive(Debug, PartialEq)]
pub struct Secret {
    raw: String,
    hangman_chars: Vec<HangmanChar>,
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
                        character: c,
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
            raw: secretstr.to_string(),
            hangman_chars: w,
            chars_to_guess,
        }
    }

    /// Process a guess and modify the game state.
    pub fn guess(&mut self, character: char) -> bool {
        let mut found = false;
        for h_char in &mut self.hangman_chars {
            if h_char.visible == false && h_char.character.eq_ignore_ascii_case(&character) {
                h_char.visible = true;
                found = true;
            }
        }

        found
    }

    /// We disclose all characters when all lives are used and the
    /// game is over.
    pub fn disclose_all(&mut self) {
        // Disclose the secret
        for hc in &mut self.hangman_chars {
            hc.visible = true;
        }
    }

    /// Method used to find out if the user has won.
    pub fn is_fully_disclosed(&self) -> bool {
        self.hangman_chars.iter().all(|c| c.visible)
    }

    /// Information used to calculate how much of the image should
    /// be disclosed.
    pub fn hidden_chars(&self) -> usize {
        self.hangman_chars.iter().filter(|hc| !hc.visible).count()
    }

    /// Used in case the secret was not guessed and we want to inject
    /// it to the dictionary again.
    pub fn to_raw_string(&self) -> String {
        self.raw.clone()
    }
}

impl fmt::Display for Secret {
    /// Graphical representation of the game state.
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut linebreak = false;
        for (n, c) in self.hangman_chars.iter().enumerate() {
            if n % LINE_WIDTH == 0 {
                linebreak = true
            };
            if n == 0 {
                linebreak = false
            };
            if linebreak && (c.character == ' ') {
                linebreak = false;
                writeln!(f, " {}", c)?
            } else {
                write!(f, " {}", c)?
            }
        }
        writeln!(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Game simulation
    #[test]
    fn test_secret() {
        let mut secret = Secret::new("_ab _cd");

        assert_eq!(secret.to_raw_string(), "_ab _cd");
        assert_eq!(format!("{}", secret), " a b   _ _\n");
        assert_eq!(secret.hidden_chars(), 2);
        assert!(!secret.is_fully_disclosed());

        secret.guess('x');

        assert_eq!(secret.to_raw_string(), "_ab _cd");
        assert_eq!(format!("{}", secret), " a b   _ _\n");
        assert_eq!(secret.hidden_chars(), 2);
        assert!(!secret.is_fully_disclosed());

        secret.guess('d');

        assert_eq!(secret.to_raw_string(), "_ab _cd");
        assert_eq!(format!("{}", secret), " a b   _ d\n");
        assert_eq!(secret.hidden_chars(), 1);
        assert!(!secret.is_fully_disclosed());

        secret.disclose_all();
        assert_eq!(secret.to_raw_string(), "_ab _cd");
        assert_eq!(format!("{}", secret), " a b   c d\n");
        assert_eq!(secret.hidden_chars(), 0);
        assert!(secret.is_fully_disclosed());
    }
}
