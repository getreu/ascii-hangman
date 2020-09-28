//! This module deals with configuration data including the management of the list of secrets

#![allow(clippy::filter_map)]
use crate::image::CONF_LINE_IDENTIFIER__CONTROL;
use crate::image::CONF_LINE_IDENTIFIER__IMAGE;
use rand::Rng;
use thiserror::Error;
//use serde::Deserialize;
use serde_derive::Deserialize;

/// Tags comment lines in the configuration file.
pub const CONF_LINE_IDENTIFIER__COMMENT: char = '#';

/// Optionally tags secret strings in config-file. Can be omitted.
pub const CONF_LINE_IDENTIFIER__WORD: char = '-';

/// A tag to enclose parts of the secret to be visible from the start, e.g.
/// "guess_-me_" will be displayed in the game as "_ _ _ _ _ - m e"
pub const CONF_LINE_SECRET_MODIFIER__VISIBLE: char = '_';

/// A tag to insert a linebreak when the secret is displayed.
pub const CONF_LINE_SECRET_MODIFIER__LINEBREAK1: char = '\n';
pub const CONF_LINE_SECRET_MODIFIER__LINEBREAK2: char = '|';

// Custom error type used expressing potential syntax errors when parsing the configuration file.
#[derive(Error, Debug)]
pub enum ConfigParseError {
    #[error(
        "Syntax error in line {line_number:?}: `{line}`\n\n\
    The game modifier must be one of the following:\n\
        :traditional-rewarding\n\
        :success-rewarding\n\n\
        Edit config file and start again.\n"
    )]
    GameModifier { line_number: usize, line: String },
    #[error(
        "Syntax error in line {line_number:?}: `{line}`\n\n\
    The first character of every non-empty line has to be one of the following:\n\
        any letter or digit (secret string),\n\
        '#' (comment line),\n\
        '-' (secret string),\n\
        '|' (ASCII-Art image) or\n\
        ':' (game modifier).\n\n\
    Edit config file and start again.\n"
    )]
    LineIdentifier { line_number: usize, line: String },
    #[error["No image data found."]]
    NoImageData,
    #[error["A config file must have a least one secret string, which is\n\
    a non-empty line starting with a letter, digit, '_' or '-'."]]
    NoSecretString,
    #[error["Could not parse the proprietary format, because this is\n\
    meant to be in (erroneous) YAML format."]]
    NotInProprietaryFormat,
    #[error(
        "Syntax error: Please follow the example below.\n\
         (The custom image is optional, it's lines start with a space.):\n\
             \t------------------------------\n\
             \tsecrets: \n\
             \t- guess me\n\
             \t- \"guess me: with colon\"\n\
             \t- line| break\n\
             \t- _disclose _partly\n\
             \n\
             \timage: |1\n\
             \t   :\n\
             \t  |_|>\n\
             \t------------------------------\n\
             {0}"
    )]
    NotInYamlFormat(#[from] serde_yaml::Error),
    #[error["First line must be: `secrets:` (no spaces allowed before)."]]
    YamlSecretsLineMissing,
}

/// We need this because `serde_yaml::Error` does not implement `PartialEq`.
/// We compare only types.
impl PartialEq for ConfigParseError {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
            && (self.to_string() == other.to_string())
    }
}

/// A dictionary holding all secret sentences from among whom one is chosen randomly at the
/// beginning of the game.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Dict {
    secrets: Vec<String>,
}

impl Dict {
    /// First try ot parse YAML, if it fails try the depreciated proprietary format and populate the dictionary
    /// with secrets.
    pub fn from(lines: &str) -> Result<Self, ConfigParseError> {
        // If both return an error, return the first one here.
        Self::from_yaml(&lines).or_else(|e| Self::from_proprietary(&lines).or(Err(e)))
    }

    /// Parse configuration file as toml data.
    pub fn from_yaml(lines: &str) -> Result<Self, ConfigParseError> {
        // Trim BOM
        let lines = lines.trim_start_matches('\u{feff}');

        for l in lines
            .lines()
            .filter(|s| !s.trim_start().starts_with('#'))
            .filter(|s| s.trim() != "")
        {
            if l.trim_end() == "secrets:" {
                break;
            } else {
                return Err(ConfigParseError::YamlSecretsLineMissing);
            }
        }

        let dict: Dict = serde_yaml::from_str(&lines)?;

        Ok(dict)
    }

    /// Parse the old configuration data format.
    fn from_proprietary(lines: &str) -> Result<Self, ConfigParseError> {
        if lines
            .lines()
            .any(|s| s.trim().starts_with("secrets:") || s.trim().starts_with("image:"))
        {
            return Err(ConfigParseError::NotInProprietaryFormat {});
        };

        let mut file_syntax_test2: Result<(), ConfigParseError> = Ok(());

        let wordlist =
          // remove Unicode BOM if present (\u{feff} has in UTF8 3 bytes).
          if lines.starts_with('\u{feff}') { &lines[3..] } else { &lines[..] }
            // interpret identifier line
            .lines()
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
                             // Lines starting alphanumerically are secret strings also.
                             // We can safely unwrap here since all empty lines had been filtered.
                             let c = l.trim().chars().next().unwrap();
                             if c.is_alphanumeric() || c == CONF_LINE_SECRET_MODIFIER__VISIBLE {
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

        file_syntax_test2?;

        if wordlist.is_empty() {
            return Err(ConfigParseError::NoSecretString {});
        }

        Ok(Dict { secrets: wordlist })
    }

    /// Chooses randomly one secret from the dictionary and removes the secret from list
    pub fn get_random_secret(&mut self) -> Option<String> {
        match self.secrets.len() {
            0 => None,
            1 => Some(self.secrets.swap_remove(0)),
            _ => {
                let mut rng = rand::thread_rng();
                let i = rng.gen_range(0, &self.secrets.len() - 1);
                Some(self.secrets.swap_remove(i))
            }
        }
    }

    /// Is there exactly one secret left?
    pub fn is_empty(&self) -> bool {
        self.secrets.is_empty()
    }

    /// Add a secret to the list.
    pub fn add(&mut self, secret: String) {
        self.secrets.push(secret);
    }
}

// ***********************

#[cfg(test)]
mod tests {
    use super::ConfigParseError;
    use super::Dict;

    /// parse all 3 data types in configuration file format
    #[test]
    fn test_new_proprietary() {
        let config: &str = "
#  comment

guess me
hang_man_
_good l_uck
:traditional-rewarding
";
        let dict = Dict::from(&config).unwrap();

        let expected = Dict {
            secrets: vec![
                "guess me".to_string(),
                "hang_man_".to_string(),
                "_good l_uck".to_string(),
            ],
        };
        assert_eq!(dict, expected);

        let config: &str = "guess me";
        let dict = Dict::from(&config);
        let expected = Ok(Dict {
            secrets: vec!["guess me".to_string()],
            // this is default
        });
        assert_eq!(dict, expected);

        // indent of comments is not allowed
        let config = "\n\n\n   # comment";
        let dict = Dict::from_proprietary(&config);
        let expected = Err(ConfigParseError::LineIdentifier {
            line_number: 4,
            line: "   # comment".to_string(),
        });
        assert_eq!(dict, expected);

        // configuration must define at least one secret
        let config = "# nothing but comment";
        let dict = Dict::from_proprietary(&config);
        let expected = Err(ConfigParseError::NoSecretString {});
        assert_eq!(dict, expected);

        let config = "one secret\n\n :traditional-rewarding";
        let dict = Dict::from_proprietary(&config);
        let expected = Err(ConfigParseError::LineIdentifier {
            line_number: 3,
            line: " :traditional-rewarding".to_string(),
        });
        assert_eq!(dict, expected);
    }

    #[test]
    fn test_new_toml() {
        let config = "# comment\nsecrets:\n  - guess me\n";
        let dict = Dict::from_yaml(&config);
        let expected = Ok(Dict {
            secrets: vec!["guess me".to_string()],
        });
        assert_eq!(dict, expected);

        let config = "# comment\nsecrets:\n- guess me\n";
        let dict = Dict::from_yaml(&config);
        let expected = Ok(Dict {
            secrets: vec!["guess me".to_string()],
        });
        assert_eq!(dict, expected);

        let config = "# comment\nsecrets:\n- 222\n";
        let dict = Dict::from_yaml(&config);
        let expected = Ok(Dict {
            secrets: vec!["222".to_string()],
        });
        assert_eq!(dict, expected);

        let config = "sxxxecrets:";
        let dict = Dict::from_yaml(&config).unwrap_err();
        assert!(matches!(dict, ConfigParseError::YamlSecretsLineMissing));

        let config = "  - guess me\nsecrets:\n";
        let dict = Dict::from_yaml(&config).unwrap_err();
        assert!(matches!(dict, ConfigParseError::YamlSecretsLineMissing));

        let config = "# comment\nsecrets:\n   guess me\n";
        let dict = Dict::from_yaml(&config).unwrap_err();
        assert!(matches!(dict, ConfigParseError::NotInYamlFormat(_)));
    }
}
