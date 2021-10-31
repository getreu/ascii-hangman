//! This module deals with configuration data including the management of the list of secrets

#![allow(clippy::manual_filter_map)]
use rand::Rng;
use thiserror::Error;
//use serde::Deserialize;
use serde_derive::Deserialize;

/// A tag to enclose parts of the secret to be visible from the start, e.g.
/// "guess_-me_" will be displayed in the game as "_ _ _ _ _ - m e"
pub const CONF_LINE_SECRET_MODIFIER_VISIBLE: char = '_';

/// A tag to insert a linebreak when the secret is displayed.
pub const CONF_LINE_SECRET_MODIFIER_LINEBREAK1: char = '\n';
pub const CONF_LINE_SECRET_MODIFIER_LINEBREAK2: char = '|';

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
    #[error["No line: `secrets:` found (no spaces allowed before)."]]
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
    /// Parse configuration file as toml data.
    pub fn from(lines: &str) -> Result<Self, ConfigParseError> {
        // Trim BOM
        let lines = lines.trim_start_matches('\u{feff}');

        if !lines
            .lines()
            .filter(|s| !s.trim_start().starts_with('#'))
            .filter(|s| s.trim() != "")
            .any(|s| s.trim_end() == "secrets:")
        {
            return Err(ConfigParseError::YamlSecretsLineMissing);
        }

        let dict: Dict = serde_yaml::from_str(lines)?;

        Ok(dict)
    }

    /// Chooses randomly one secret from the dictionary and removes the secret from list
    pub fn get_random_secret(&mut self) -> Option<String> {
        match self.secrets.len() {
            0 => None,
            1 => Some(self.secrets.swap_remove(0)),
            _ => {
                let mut rng = rand::thread_rng();
                let i = rng.gen_range(0..self.secrets.len());
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
    fn test_from() {
        let config: &str = "
#  comment
secrets:
- guess me
- hang_man_
- _good l_uck

traditional: true
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
        let config = "# comment\nsecrets:\n  - guess me\n";
        let dict = Dict::from(&config);
        let expected = Ok(Dict {
            secrets: vec!["guess me".to_string()],
        });
        assert_eq!(dict, expected);

        let config = "# comment\nsecrets:\n- guess me\n";
        let dict = Dict::from(&config);
        let expected = Ok(Dict {
            secrets: vec!["guess me".to_string()],
        });
        assert_eq!(dict, expected);

        let config = "# comment\nsecrets:\n- 222\n";
        let dict = Dict::from(&config);
        let expected = Ok(Dict {
            secrets: vec!["222".to_string()],
        });
        assert_eq!(dict, expected);

        let config = "sxxxecrets:";
        let dict = Dict::from(&config).unwrap_err();
        assert!(matches!(dict, ConfigParseError::YamlSecretsLineMissing));

        let config = "  - guess me\nsecrets:\n";
        let dict = Dict::from(&config).unwrap_err();
        assert!(matches!(dict, ConfigParseError::YamlSecretsLineMissing));

        let config = "# comment\nsecrets:\n   guess me\n";
        let dict = Dict::from(&config).unwrap_err();
        assert!(matches!(dict, ConfigParseError::NotInYamlFormat(_)));
    }
}
