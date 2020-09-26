use crate::dictionary::CONF_LINE_SECRET_MODIFIER__LINEBREAK;
use crate::dictionary::CONF_LINE_SECRET_MODIFIER__VISIBLE;
use std::fmt;

/// Defines the line-break position when displaying the secret string.
const LINE_WIDTH: usize = 20;

/// The character type.
#[derive(Clone, Debug, PartialEq)]
enum HangmanCharType {
    Visible,
    Hidden,
    Formatter,
    Ignored,
}

/// One character of the secret string.
#[derive(Clone, Debug, PartialEq)]
struct HangmanChar {
    character: char,
    chartype: HangmanCharType,
}

/// The secret
#[derive(Debug, PartialEq)]
pub struct Secret {
    hangman_chars: Vec<HangmanChar>,
    pub chars_to_guess: usize,
}

impl Secret {
    /// Constructor.
    pub fn new(secretstr: &str) -> Self {
        // parse `secretsstr`, flip 'visible' every CONF_LINE_SECRET_MODIFIER__VISIBLE
        let mut whitespace_on = false;
        let mut visible_on = false;
        let w: Vec<HangmanChar> = secretstr
            .chars()
            // For every `_` found flip `visible_on`.
            // CONF_LINE_SECRET_MODIFIER__LINEBREAK found, set `whitespace_on=true`.
            // Non whitespace found, set `whitespace_on=false`.
            .map(|c| {
                if whitespace_on && !c.is_whitespace() {
                    whitespace_on = false;
                };
                if c == CONF_LINE_SECRET_MODIFIER__LINEBREAK {
                    whitespace_on = true;
                };
                visible_on ^= c == CONF_LINE_SECRET_MODIFIER__VISIBLE;
                let ct = match (c, visible_on, whitespace_on) {
                    (CONF_LINE_SECRET_MODIFIER__VISIBLE, _, _) => HangmanCharType::Formatter,
                    (CONF_LINE_SECRET_MODIFIER__LINEBREAK, _, _) => HangmanCharType::Formatter,
                    (_, _, true) => HangmanCharType::Ignored,
                    (_, true, false) => HangmanCharType::Visible,
                    (_, false, false) => HangmanCharType::Hidden,
                };
                HangmanChar {
                    character: c,
                    chartype: ct,
                }
            })
            .collect();

        let chars_to_guess = w
            .iter()
            .filter(|hc| matches!(hc.chartype, HangmanCharType::Hidden))
            .count();

        Self {
            hangman_chars: w,
            chars_to_guess,
        }
    }

    /// Process a guess and modify the game state.
    pub fn guess(&mut self, character: char) -> bool {
        let mut found = false;
        for h_char in &mut self.hangman_chars {
            if matches!(h_char.chartype, HangmanCharType::Hidden)
                && h_char.character.eq_ignore_ascii_case(&character)
            {
                h_char.chartype = HangmanCharType::Visible;
                found = true;
            }
        }

        found
    }

    /// We disclose all characters when all lives are used and the
    /// game is over.
    pub fn disclose_all(&mut self) {
        // Disclose the secret
        for hc in &mut self
            .hangman_chars
            .iter_mut()
            .filter(|c| matches!(c.chartype, HangmanCharType::Hidden))
        {
            hc.chartype = HangmanCharType::Visible;
        }
    }

    /// Method used to find out if the user has won.
    pub fn is_fully_disclosed(&self) -> bool {
        !self
            .hangman_chars
            .iter()
            .filter(|&c| matches!(c.chartype, HangmanCharType::Hidden))
            .any(|c| matches!(c.chartype, HangmanCharType::Hidden))
    }

    /// Information used to calculate how much of the image should
    /// be disclosed.
    pub fn hidden_chars(&self) -> usize {
        self.hangman_chars
            .iter()
            .filter(|hc| matches!(hc.chartype, HangmanCharType::Hidden))
            .count()
    }

    /// Used in case the secret was not guessed and we want to inject
    /// it to the dictionary again.
    pub fn to_raw_string(&self) -> String {
        self.hangman_chars.iter().map(|hc| hc.character).collect()
    }
}

impl fmt::Display for Secret {
    /// Graphical representation of the secret taking into account the
    /// game state.
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut linebreak = false;
        let mut n = 1;
        for c in self.hangman_chars.iter() {
            if n >= LINE_WIDTH {
                linebreak = true
            };
            if matches!(c.chartype, HangmanCharType::Formatter)
                && c.character == CONF_LINE_SECRET_MODIFIER__LINEBREAK
            {
                linebreak = true
            };

            if linebreak
                && (!matches!(c.chartype, HangmanCharType::Formatter) && (c.character == ' ')
                    || (matches!(c.chartype, HangmanCharType::Formatter)
                        && c.character == CONF_LINE_SECRET_MODIFIER__LINEBREAK))
            {
                linebreak = false;
                n = 0;
                writeln!(f, "")?;
            } else {
                match c.chartype {
                    HangmanCharType::Visible => {
                        write!(f, " {}", c.character)?;
                        n += 1;
                    }
                    HangmanCharType::Hidden => {
                        write!(f, " _")?;
                        n += 1;
                    }
                    HangmanCharType::Formatter => {}
                    HangmanCharType::Ignored => {}
                };
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
    #[test]
    fn test_secret_linebreak() {
        let mut secret = Secret::new("_abc|def _hij|klm");
        assert_eq!(secret.to_string(), " a b c\n d e f   _ _ _\n _ _ _\n");
        assert_eq!(secret.to_raw_string(), "_abc|def _hij|klm");
        assert_eq!(secret.hidden_chars(), 6);
        assert!(!secret.is_fully_disclosed());

        secret.disclose_all();
        assert_eq!(secret.to_string(), " a b c\n d e f   h i j\n k l m\n");
        assert_eq!(secret.hidden_chars(), 0);
        assert!(secret.is_fully_disclosed());
        assert_eq!(secret.to_raw_string(), "_abc|def _hij|klm");

        let secret = Secret::new("_123456789012345 789012345 789012_");
        assert_eq!(
            secret.to_string(),
            " 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5   7 8 9 0 1 2 3 4 5\n 7 8 9 0 1 2\n"
        );

        let secret = Secret::new("_abc|  def _hij| \n  klm");
        assert_eq!(secret.to_string(), " a b c\n d e f   _ _ _\n _ _ _\n");
        assert_eq!(secret.to_raw_string(), "_abc|  def _hij| \n  klm");
        assert_eq!(secret.hidden_chars(), 6);
        assert!(!secret.is_fully_disclosed());
    }
}
