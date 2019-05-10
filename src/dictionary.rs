#![allow(clippy::filter_map)]
extern crate rand;
use crate::image::CONF_LINE_IDENTIFIER__IMAGE;
use rand::seq::SliceRandom;
use rand::thread_rng;


// the default can be changed by one of the following switches
const DEFAULT_REWARDING_SCHEME: RewardingScheme = RewardingScheme::UnhideWhenGuessedChar;

// Keyword to switch rewarding scheme
// :traditional-rewarding
const UNHIDE_WHEN_LOST_LIVE_IDENTIFIER: &str = "traditional-rewarding";

// Keyword to switch rewarding scheme
// :success-rewarding
const UNHIDE_WHEN_GUESSED_CHAR_IDENTIFIER: &str = "success-rewarding";

// comments in config file start with
pub const CONF_LINE_IDENTIFIER__COMMENT: char = '#';

// comands in config-file start with
pub const CONF_LINE_IDENTIFIER__CONTROL: char = ':';

// secret strings in config-file start with
pub const CONF_LINE_IDENTIFIER__WORD: char = '-';

// a modifier tagging parts of the string to be visible from the start, e.g.
// "guess_-me_: will be shown as "_ _ _ _ _ - m e"
pub const CONF_LINE_WORD_MODIFIER__VISIBLE: char = '_';


custom_error! {pub ConfigParseError
    GameModifier{line_number: usize, line: String}   = "
Syntax error in line 
{line_number}:  \"{line}\"

The game modifier must be one of the following:
    :traditional-rewarding
    :success-rewarding

Edit config file and start again.\n",
    LineIdentifier{line_number: usize, line: String} = "
Syntax error in line 
{line_number}:  \"{line}\"

The first character of every non-empty line has to be one of the following:
    any letter or digit (secret string), 
    '#' (comment line), 
    '-' (secret string), 
    '|' (ASCII-Art image) or 
    ':' (game modifier).

Edit config file and start again.\n",
    NoSecretString{} = "
A config file must have a least one secret string, which is 
a non-empty line starting with a letter, digit, '_' or '-'.
",
}


#[derive(Debug, PartialEq)]
pub enum RewardingScheme {
    UnhideWhenLostLife,
    UnhideWhenGuessedChar,
}

#[derive(Debug)]
pub struct Dict {
    wordlist: Vec<String>,
    pub rewarding_scheme: RewardingScheme,
}

impl Dict {
    pub fn new(lines: &str) -> Result<Self, ConfigParseError> {
        let mut rewarding_scheme = DEFAULT_REWARDING_SCHEME;
        let mut file_syntax_test1: Result<(), ConfigParseError> = Ok(());
        let mut file_syntax_test2: Result<(), ConfigParseError> = Ok(());

        let wordlist =
          // remove Unicode BOM if present (\u{feff} has in UTF8 3 bytes).
          if lines.starts_with('\u{feff}') { &lines[3..] } else { &lines[..] }
            // interpret identifier line
            .lines()
            .enumerate()
            .filter(|&(n,l)| {
                if l.starts_with(CONF_LINE_IDENTIFIER__CONTROL) {
                    if l[1..].trim() == UNHIDE_WHEN_LOST_LIVE_IDENTIFIER {
                        rewarding_scheme = RewardingScheme::UnhideWhenLostLife;
                        false
                    }
                    else if l[1..].trim() == UNHIDE_WHEN_GUESSED_CHAR_IDENTIFIER {
                        rewarding_scheme = RewardingScheme::UnhideWhenGuessedChar;
                        false
                    }
                    else {
                        // we only save the first error
                        if file_syntax_test1.is_ok() {
                            file_syntax_test1 = Err(ConfigParseError::GameModifier {
                                line_number: n+1, line: l.to_string() });
                        };
                        false
                    }
                } else {
                    true
                }
            })
            .filter(|&(_,l)|!( l.trim().is_empty() ||
                          l.starts_with(CONF_LINE_IDENTIFIER__COMMENT) ||
                          l.starts_with(CONF_LINE_IDENTIFIER__CONTROL) ||
                          l.starts_with(CONF_LINE_IDENTIFIER__IMAGE)
                        )
            )
            .map(|(n,l)| if l.starts_with(CONF_LINE_IDENTIFIER__WORD) {
                             l[1..].trim().to_string()
                        } else {
                             // Lines starting alphanumericly are secret strings also.
                             // We can safely unwrap here since all empty lines had been filtered.
                             let c = l.chars().next().unwrap();
                             if c.is_alphanumeric() || c == CONF_LINE_WORD_MODIFIER__VISIBLE {
                                l.trim().to_string()
                             } else {
                                 // we only save the first error
                                 if file_syntax_test2.is_ok() {
                                    file_syntax_test2 = Err(ConfigParseError::LineIdentifier {
                                        line_number: n+1, line: l.to_string() });
                                 };
                                // This will never be used but we have to return something
                                "".to_string()
                             }
                        }
            )
            .collect::<Vec<String>>();

        if file_syntax_test1.is_err() {
            return Err(file_syntax_test1.unwrap_err());
        };
        if file_syntax_test2.is_err() {
            return Err(file_syntax_test2.unwrap_err());
        };

        if wordlist.is_empty() {
            return Err(ConfigParseError::NoSecretString {});
        }

        Ok(Dict {
            wordlist,
            rewarding_scheme,
        })
    }

    pub fn get_random_word(&self) -> String {
        let mut rng = thread_rng();
        (&self.wordlist).choose(&mut rng).unwrap().to_string()
    }
}
