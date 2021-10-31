//! This module contains all the logic dealing with images:
//! Parsing the config file data, shuffling pixels of big images,
//! ordering pixels of small images and sorting the signatures to the end.

use crate::ascii_art::DEFAULT_IMAGES;
use crate::ascii_art::IMAGE_KNOWN_SIGNATURES;
use crate::dictionary::ConfigParseError;
use crate::game::Game;
use crate::LIVES;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde_derive::Deserialize;
use std::cmp::{Ord, Ordering};
use std::fmt;

/// Identifier tagging image data in configuration files.
pub const CONF_LINE_IDENTIFIER__IMAGE: char = '|';

/// Tags control common lines in the configuration file.
pub const CONF_LINE_IDENTIFIER__CONTROL: char = ':';

/// Default game mode. Can be changed in the configuration file.
const DEFAULT_REWARDING_SCHEME: RewardingScheme = RewardingScheme::UnhideWhenGuessedChar;

/// Keyword in the configuration file to switch rewarding scheme in the enum `RewardingScheme`
/// to `UnHideWhenLostLife`
const UNHIDE_WHEN_LOST_LIVE_IDENTIFIER: &str = "traditional-rewarding";

/// Keyword in the configuration file to switch rewarding scheme in the enum `RewardingScheme`
/// to `UnHideWhenGuessedChar`
const UNHIDE_WHEN_GUESSED_CHAR_IDENTIFIER: &str = "success-rewarding";

/// Threshold to decide from how many characters on the images is considered to be "big".
/// Big images are disclosed with another algorithm.
/// This is just big enough that the gallow image stays small.
const BIG_IMAGE: usize = 60; // sort algorithm <-> random algorithm

/// A game mode defining how the ASCII-art image will be disclosed progressively.
#[derive(Clone, Debug, PartialEq)]
pub enum RewardingScheme {
    /// Game mode that is used together with the traditional gallows image (the gallows image
    /// is not build in, but can be added in the configuration file. The image is disclosed
    /// piecemeal after each wrong guess.
    UnhideWhenLostLife,
    /// Default game mode. The image is disclosed piecemeal after each right guess.
    UnhideWhenGuessedChar,
}
/// One character of the ASCII art image.
#[derive(Eq, PartialEq, Debug, Copy, Clone)] //omitting Ord
pub struct ImChar {
    pub point: (u8, u8),
    pub code: char,
}

/// Format an image character.
impl fmt::Display for ImChar {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.code)
    }
}

impl PartialOrd for ImChar {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
/// Ord enables us to v.sort() the image characters.
impl Ord for ImChar {
    /// Compares to ImChar.
    /// Points near the left lower corner are small.
    fn cmp(&self, other: &Self) -> Ordering {
        fn weight(ic: &ImChar) -> isize {
            let &ImChar { point: (x, y), .. } = ic;
            // points near the lower left corner are light
            x as isize - y as isize
        }
        weight(self).cmp(&weight(other))
    }
}

#[derive(Clone, Debug, PartialEq)]
/// An ASCII-art image.
pub struct Image {
    pub ichars: Vec<ImChar>,
    pub dimension: (u8, u8),
    pub visible_points: usize,
    pub rewarding_scheme: RewardingScheme,
}

/// Format an image.
impl fmt::Display for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let x_max = self.dimension.0 as usize;
        let y_max = self.dimension.1 as usize;

        let mut i = vec![' '; ((x_max + 1) * y_max) as usize];
        for y in 0..y_max {
            i[((x_max + 1) * y + x_max) as usize] = '\n';
        }

        for ic in self.ichars.iter().take(self.visible_points) {
            let &ImChar {
                point: (x, y),
                code,
            } = ic;
            i[(x as usize + y as usize * (x_max + 1))] = code;
        }

        write!(f, "{}", i.into_iter().collect::<String>())
    }
}

impl Image {
    /// Returns a random built-in image.
    pub fn new() -> Result<Self, ConfigParseError> {
        let mut rng = thread_rng();
        Self::from_yaml((DEFAULT_IMAGES).choose(&mut rng).unwrap())
    }

    /// First try ot parse YAML, if it fails try the depreciated proprietary format and
    /// read the image data.
    pub fn from_formatted(input: &str) -> Result<Self, ConfigParseError> {
        // If both return an error, return the first one here.
        Self::from_yaml(input).or_else(|e| Self::from_proprietary(input).or(Err(e)))
    }

    /// Constructor reading image data from YAML configuration files.
    pub fn from_yaml(input: &str) -> Result<Self, ConfigParseError> {
        #[derive(Debug, PartialEq, Deserialize)]
        pub struct RawImage {
            image: Option<String>,
            traditional: Option<bool>,
        }

        let input = input.trim_start_matches('\u{feff}');

        let raw: RawImage = serde_yaml::from_str(input)?;

        let (image, rewarding_scheme) = match raw {
            RawImage { image: None, .. } => return Err(ConfigParseError::NoImageData),
            RawImage {
                image: Some(i),
                traditional: None,
            } => (i, RewardingScheme::UnhideWhenGuessedChar),
            RawImage {
                image: Some(i),
                traditional: Some(r),
            } => (
                i,
                if r {
                    RewardingScheme::UnhideWhenLostLife
                } else {
                    RewardingScheme::UnhideWhenGuessedChar
                },
            ),
        };

        Self::from(&image, rewarding_scheme)
    }

    /// Constructor reading image data from proprietary configuration files.
    pub fn from_proprietary(string: &str) -> Result<Self, ConfigParseError> {
        let mut v: Vec<ImChar> = Vec::new();
        let mut rewarding_scheme = DEFAULT_REWARDING_SCHEME;
        let mut file_syntax_test1: Result<(), ConfigParseError> = Ok(());

        for (y, line) in string
            // split in lines
            .lines()
            .enumerate()
            .filter(|&(n, l)| {
                if l.starts_with(CONF_LINE_IDENTIFIER__CONTROL) {
                    if l[1..].trim() == UNHIDE_WHEN_LOST_LIVE_IDENTIFIER {
                        rewarding_scheme = RewardingScheme::UnhideWhenLostLife;
                        false
                    } else if l[1..].trim() == UNHIDE_WHEN_GUESSED_CHAR_IDENTIFIER {
                        rewarding_scheme = RewardingScheme::UnhideWhenGuessedChar;
                        false
                    } else {
                        // we only save the first error
                        if file_syntax_test1.is_ok() {
                            file_syntax_test1 = Err(ConfigParseError::GameModifier {
                                line_number: n + 1,
                                line: l.to_string(),
                            });
                        };
                        false
                    }
                } else {
                    true
                }
            })
            .map(|(_, l)| l)
            // consider only lines starting with '|'
            .filter(|&l| l.starts_with(CONF_LINE_IDENTIFIER__IMAGE))
            .enumerate()
        //.inspect(|&(n,l)| println!("line {:?}: {:?} ", n,l))
        {
            let mut ii: Vec<_> = line
                .char_indices()
                // skip first char '|'
                .skip(1)
                // consider only chars != ' '
                .filter(|&(_, c)| c != ' ')
                // save in ImChar object
                .map(|(x, c)| ImChar {
                    // subtract the char we have skipped before
                    point: ((x - 1) as u8, y as u8),
                    code: c,
                })
                .collect();
            v.append(&mut ii);
        }

        file_syntax_test1?;

        // find dimensions
        let dimension = if !v.is_empty() {
            let mut x_max = 0;
            let mut y_max = 0;

            for i in &v {
                let &ImChar { point: (x, y), .. } = i;
                if x > x_max {
                    x_max = x
                };
                if y > y_max {
                    y_max = y
                };
            }
            // we know there is at least one char
            (x_max + 1, y_max + 1)
        } else {
            (0, 0)
        };

        // order points
        let v_len = v.len();
        if v_len <= BIG_IMAGE {
            v.sort(); // Sort algorithm, see "impl Ord for ImageChar"
        } else {
            let mut rng = thread_rng();
            (&mut v).shuffle(&mut rng); // points appear randomly.
        }

        if v.is_empty() {
            Err(ConfigParseError::NoImageData)
        } else {
            Ok(Self {
                ichars: v,
                dimension,
                visible_points: v_len,
                rewarding_scheme,
            })
        }
    }

    #[inline]
    /// This constructor takes a pure ASCII, non-escaped, multiline image string.
    pub fn from(image: &str, rewarding_scheme: RewardingScheme) -> Result<Self, ConfigParseError> {
        let mut ascii: Vec<ImChar> = Vec::new();
        let mut signature: Vec<ImChar> = Vec::new();

        // Create a string of `' '` with length of longest `IMAGE_KNOWN_SIGNATURES`.
        let mut spaces = String::new();
        let longest = IMAGE_KNOWN_SIGNATURES
            .iter()
            .map(|s| s.len())
            .max()
            .unwrap();
        for _ in 0..longest {
            spaces.push(' ');
        }

        for (y, line) in image.lines().enumerate() {
            let mut ascii_line = line.to_owned();
            for sig in IMAGE_KNOWN_SIGNATURES {
                // `spaces` has the same length than `sig`.
                let short_spaces = &spaces[..sig.len()];
                debug_assert_eq!(sig.len(), short_spaces.len());
                ascii_line = ascii_line.replace(sig, short_spaces);
            }
            debug_assert_eq!(line.len(), ascii_line.len());

            // Generate `ImChar` pixel from `ascii_line`.
            let mut ii: Vec<_> = ascii_line
                .char_indices()
                // consider only chars != ' '
                .filter(|&(_, c)| c != ' ')
                // save in ImChar object
                .map(|(x, c)| ImChar {
                    point: ((x) as u8, y as u8),
                    code: c,
                })
                .collect();
            ascii.append(&mut ii);

            // Check what we have changed and generate
            // `signature_line`.
            let mut signature_line = String::new();
            for (l, a) in line.chars().zip(ascii_line.chars()) {
                if l == a {
                    // Nothing changed here.
                    signature_line.push(' ');
                } else {
                    signature_line.push(l);
                }
            }
            debug_assert_eq!(signature_line.chars().count(), ascii_line.chars().count());

            // Generate `ImChar` pixel from `signature_line`.
            let mut ii: Vec<_> = signature_line
                .char_indices()
                // consider only chars != ' '
                .filter(|&(_, c)| c != ' ')
                // save in ImChar object
                .map(|(x, c)| ImChar {
                    point: ((x) as u8, y as u8),
                    code: c,
                })
                .collect();
            signature.append(&mut ii);
        }

        // Order or shuffle pixel in `ascii`
        if ascii.len() <= BIG_IMAGE {
            ascii.sort(); // Sort algorithm, see "impl Ord for ImageChar"
        } else {
            let mut rng = thread_rng();
            (&mut ascii).shuffle(&mut rng); // points appear randomly.
        }

        // Append `signatures` at the end of `ascii`.
        ascii.append(&mut signature);

        // Find the dimensions of the whole.
        let dimension = if !ascii.is_empty() {
            let mut x_max = 0;
            let mut y_max = 0;

            for i in &ascii {
                let &ImChar { point: (x, y), .. } = i;
                if x > x_max {
                    x_max = x
                };
                if y > y_max {
                    y_max = y
                };
            }
            // We know that there is at least one char.
            (x_max + 1, y_max + 1)
        } else {
            (0, 0)
        };

        // Find the number of pixels.
        let visible_points = ascii.len();

        if ascii.is_empty() {
            Err(ConfigParseError::NoImageData)
        } else {
            Ok(Self {
                ichars: ascii,
                dimension,
                visible_points,
                rewarding_scheme,
            })
        }
    }

    /// Discloses parts of the image according to the course of the play.
    pub fn update(&mut self, game: &Game) {
        match self.rewarding_scheme {
            RewardingScheme::UnhideWhenGuessedChar => {
                if game.lifes != 0 {
                    self.hide((game.secret.hidden_chars(), game.secret.chars_to_guess()));
                }
            }
            RewardingScheme::UnhideWhenLostLife => {
                self.hide((game.lifes as usize, LIVES as usize));
            }
        };
    }

    /// Sets how much of the image will be disclosed next time the image is rendered.
    fn hide(&mut self, fraction: (usize, usize)) {
        let l = self.ichars.len();

        let as_points = |(n, d)| (5 * l * (d - n) as usize / d as usize + l) / 6;

        // silently ignore division by zero
        if fraction.1 > 0 {
            self.visible_points = as_points(fraction);
        };
    }
}

// *******************************

#[cfg(test)]
mod tests {
    use super::DEFAULT_REWARDING_SCHEME;
    use super::{ImChar, Image};
    use crate::dictionary::ConfigParseError;

    /// Test image parsing of configuration file data
    #[test]
    fn test_image_parser_syntax() {
        let config: &str = r#"
|ab
|cd"#;
        let image = Image::from_proprietary(&config);
        //println!("{:?}",image);
        let expected = Ok(Image {
            ichars: [
                ImChar {
                    point: (0, 0),
                    code: 'a',
                },
                ImChar {
                    point: (0, 1),
                    code: 'c',
                },
                ImChar {
                    point: (1, 0),
                    code: 'b',
                },
                ImChar {
                    point: (1, 1),
                    code: 'd',
                },
            ]
            .to_vec(),
            dimension: (2, 2),
            visible_points: 4,
            rewarding_scheme: DEFAULT_REWARDING_SCHEME,
        });

        assert!(image == expected);
    }

    /// Is non image data ignored?
    #[test]
    fn test_image_parser_syntax_ignore() {
        let config: &str = r#"
|/\
\/"#;
        let image = Image::from_proprietary(&config).unwrap();
        //println!("{:?}",image);
        let expected = Image {
            ichars: [
                ImChar {
                    point: (0, 0),
                    code: '/',
                },
                ImChar {
                    point: (1, 0),
                    code: '\\',
                },
            ]
            .to_vec(),
            dimension: (2, 1),
            visible_points: 2,
            rewarding_scheme: DEFAULT_REWARDING_SCHEME,
        };

        assert_eq!(image, expected);
    }

    #[test]
    fn test_image_renderer() {
        let config: &str = r#"
|>o)
|(_>   <o)
|      (_>
"#;
        let expected: &str = ">o)      \n(_>   <o)\n      (_>\n";
        let image = Image::from_proprietary(&config).unwrap();

        assert!(image.visible_points > 0);
        assert_eq!(format!("{}", image), expected);
    }

    #[test]
    fn test_image_renderer_yaml() {
        let config: &str = r#"image: |1
 >o)
 (_>   <o)
       (_>
"#;
        let expected: &str = ">o)      \n(_>   <o)\n      (_>\n";
        let image = Image::from_yaml(&config).unwrap();

        assert!(image.visible_points > 0);
        assert_eq!(format!("{}", image), expected);
    }

    #[test]
    fn test_image_parser_built_in_image() {
        let config: &str = "this is no image";
        let image = Image::from_yaml(&config).unwrap_err();
        //println!("{:?}",image);

        assert!(matches!(image, ConfigParseError::NotInYamlFormat(_)));
    }

    /// disclose image progressively
    #[test]
    fn test_proprietery_image_parser_disclose() {
        let config: &str = "|abcde\n|f";
        let mut image = Image::from_proprietary(&config).unwrap();
        //println!("{:?}",image);
        let expected = Image {
            ichars: [
                ImChar {
                    point: (0, 0),
                    code: 'a',
                },
                ImChar {
                    point: (0, 1),
                    code: 'f',
                },
                ImChar {
                    point: (1, 0),
                    code: 'b',
                },
                ImChar {
                    point: (2, 0),
                    code: 'c',
                },
                ImChar {
                    point: (3, 0),
                    code: 'd',
                },
                ImChar {
                    point: (4, 0),
                    code: 'e',
                },
            ]
            .to_vec(),
            dimension: (5, 2),
            visible_points: 6,
            rewarding_scheme: DEFAULT_REWARDING_SCHEME,
        };
        assert_eq!(image, expected);

        image.hide((6, 6));
        assert_eq!(image.visible_points, 1);

        image.hide((2, 6));
        assert_eq!(image.visible_points, 4);

        image.hide((0, 6));
        assert_eq!(image.visible_points, 6);
    }

    #[test]
    fn test_yaml_image_parser_disclose() {
        //
        // Test yaml.
        let config: &str = "image: |1\n abcde\n f";
        let mut image = Image::from_yaml(&config).unwrap();
        //println!("{:?}",image);
        let expected = Image {
            ichars: [
                ImChar {
                    point: (0, 0),
                    code: 'a',
                },
                ImChar {
                    point: (0, 1),
                    code: 'f',
                },
                ImChar {
                    point: (1, 0),
                    code: 'b',
                },
                ImChar {
                    point: (2, 0),
                    code: 'c',
                },
                ImChar {
                    point: (3, 0),
                    code: 'd',
                },
                ImChar {
                    point: (4, 0),
                    code: 'e',
                },
            ]
            .to_vec(),
            dimension: (5, 2),
            visible_points: 6,
            rewarding_scheme: DEFAULT_REWARDING_SCHEME,
        };
        assert_eq!(image, expected);

        image.hide((6, 6));
        assert_eq!(image.visible_points, 1);

        image.hide((2, 6));
        assert_eq!(image.visible_points, 4);

        image.hide((0, 6));
        assert_eq!(image.visible_points, 6);
    }

    #[test]
    fn disclose_signature_last() {
        let image_str = "jensA\nBlisC";
        let image = Image::from(&image_str, DEFAULT_REWARDING_SCHEME).unwrap();
        //println!("{:?}",image);
        let expected = Image {
            ichars: [
                // This is the ASCII part of the image.
                ImChar {
                    point: (0, 1),
                    code: 'B',
                },
                ImChar {
                    point: (4, 0),
                    code: 'A',
                },
                ImChar {
                    point: (4, 1),
                    code: 'C',
                },
                // From here on, we see only signatures.
                ImChar {
                    point: (0, 0),
                    code: 'j', // was `j`
                },
                ImChar {
                    point: (1, 0),
                    code: 'e', // was `j`
                },
                ImChar {
                    point: (2, 0),
                    code: 'n',
                },
                ImChar {
                    point: (3, 0),
                    code: 's',
                },
                ImChar {
                    point: (1, 1),
                    code: 'l',
                },
                ImChar {
                    point: (2, 1),
                    code: 'i',
                },
                ImChar {
                    point: (3, 1),
                    code: 's',
                },
            ]
            .to_vec(),
            dimension: (5, 2),
            visible_points: 10,
            rewarding_scheme: DEFAULT_REWARDING_SCHEME,
        };
        assert_eq!(image, expected);
    }

    /// test game modifier spelling
    #[test]
    fn test_image_parser_error_misspelled() {
        let config = "\n\n:traditional-rewardXing";
        let dict = Image::from_proprietary(&config);
        let expected = Err(ConfigParseError::GameModifier {
            line_number: 3,
            line: ":traditional-rewardXing".to_string(),
        });
        assert_eq!(dict, expected);
    }
}
