#![allow(clippy::filter_map)]
extern crate rand;
use crate::image::CONF_LINE_IDENTIFIER__IMAGE;
use rand::seq::SliceRandom;
use rand::thread_rng;

// Config file syntax error message
pub const CONF_SYNTAX_ERROR: &str = "

SYNTAX ERROR in config file!
The first character of every line has to be one of the following:
    any letter or digit (guessing string), 
    '#' (comment line), 
    '-' (guessing string), 
    '|' (ASCII-Art image) or 
    ':' (game modifier).

Edit config file and start again.\n";

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

// guessing strings in config-file start with
pub const CONF_LINE_IDENTIFIER__WORD: char = '-';

// a modifier tagging parts of the string to be visible from the start, e.g.
// "guess_-me_: will be shown as "_ _ _ _ _ - m e"
pub const CONF_LINE_WORD_MODIFIER__VISIBLE: char = '_';

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
    pub fn len(&self) -> usize {
        self.wordlist.len()
    }

    pub fn new(lines: &str) -> Self {
        let mut rewarding_scheme = DEFAULT_REWARDING_SCHEME;
        let wordlist =
          // remove Unicode BOM if present (\u{feff} has in UTF8 3 bytes).
          if lines.starts_with('\u{feff}') { &lines[3..] } else { &lines[..] }
            // interpret identifier line
            .lines()
            .filter_map(|l| {
                if l.starts_with(CONF_LINE_IDENTIFIER__CONTROL) {
                    if l[1..].trim().contains(UNHIDE_WHEN_LOST_LIVE_IDENTIFIER) {
                        rewarding_scheme = RewardingScheme::UnhideWhenLostLife;
                    }
                    if l[1..].trim().contains(UNHIDE_WHEN_GUESSED_CHAR_IDENTIFIER) {
                        rewarding_scheme = RewardingScheme::UnhideWhenGuessedChar;
                    }
                    None
                } else {
                    Some(l)
                }
            })
            .enumerate()
            .filter(|&(_,l)|!( l.trim().is_empty() ||
                          l.starts_with(CONF_LINE_IDENTIFIER__COMMENT) ||
                          l.starts_with(CONF_LINE_IDENTIFIER__CONTROL) ||
                          l.starts_with(CONF_LINE_IDENTIFIER__IMAGE)
                        )
            )
            .map(|(n,l)| if l.starts_with(CONF_LINE_IDENTIFIER__WORD) {
                             l[1..].trim().to_string()
                        } else {
                             // Lines starting alphanumeric are guessing strings also.
                             // We can safely unwrap here since all empty lines had been filtered.
                             let c = l.chars().next().unwrap();
                             if c.is_alphanumeric() || c == CONF_LINE_WORD_MODIFIER__VISIBLE {
                                 l.trim().to_string()
                             } else {
                             panic!("{}\nError in line: {}: \"{}\"\n\n",
                                     CONF_SYNTAX_ERROR, n+1, l)
                             }
                        }
            )
            .collect();
        Dict {
            wordlist,
            rewarding_scheme,
        }
    }

    pub fn get_random_word(&self) -> String {
        let mut rng = thread_rng();
        (&self.wordlist).choose(&mut rng).unwrap().to_string()
    }
}
