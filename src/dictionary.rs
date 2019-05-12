//! This module deals with configuration data including the management of the list of secrets

#![allow(clippy::filter_map)]
extern crate rand;
use crate::image::CONF_LINE_IDENTIFIER__IMAGE;
use rand::seq::SliceRandom;
use rand::thread_rng;

/// Default game mode. Can be changed in the configuration file.
const DEFAULT_REWARDING_SCHEME: RewardingScheme = RewardingScheme::UnhideWhenGuessedChar;

/// Keyword in the configuration file to switch rewarding scheme in the enum `RewardingScheme`
/// to `UnHideWhenLostLife`
const UNHIDE_WHEN_LOST_LIVE_IDENTIFIER: &str = "traditional-rewarding";

/// Keyword in the configuration file to switch rewarding scheme in the enum `RewardingScheme`
/// to `UnHideWhenGuessedChar`
const UNHIDE_WHEN_GUESSED_CHAR_IDENTIFIER: &str = "success-rewarding";

/// Tags comment lines in the configuration file.
pub const CONF_LINE_IDENTIFIER__COMMENT: char = '#';

/// Tags control comman lines in the configuration file.
pub const CONF_LINE_IDENTIFIER__CONTROL: char = ':';

/// Optionally tags secret strings in config-file. Can be omitted.
pub const CONF_LINE_IDENTIFIER__WORD: char = '-';

/// A tag to enclose parts of the secret to be visible from the start, e.g.
/// "guess_-me_" will be displayed in the game as "_ _ _ _ _ - m e"
pub const CONF_LINE_WORD_MODIFIER__VISIBLE: char = '_';

/// Custom error type used expressing potential syntax errors when parsing the configuration file.
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

/// A game mode defining how the ASCII-art image will be disclosed progressively.
#[derive(Debug, PartialEq)]
pub enum RewardingScheme {
    /// Game mode that is used together with the traditional gallows image (the gallows image
    /// is not build in, but can be add in the configuration file. The image is disclosed
    /// piecemeal after each wrong guess.
    UnhideWhenLostLife,
    /// Default game mode. The image is disclosed piecemeal after each right guess.
    UnhideWhenGuessedChar,
}

/// A dictionary holding all secret sentences from among whom one is chosen randomly at the
/// beginning of the game.
#[derive(Debug)]
pub struct Dict {
    wordlist: Vec<String>,
    pub rewarding_scheme: RewardingScheme,
}

impl Dict {
    /// Parses the configuration data, sets game modifier variables and populates the dictionary
    /// with secrets.
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
                             let c = l.trim().chars().next().unwrap();
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

    /// Chooses randomly one secret from the dictionary.
    pub fn get_random_word(&self) -> String {
        let mut rng = thread_rng();
        (&self.wordlist).choose(&mut rng).unwrap().to_string()
    }
}
