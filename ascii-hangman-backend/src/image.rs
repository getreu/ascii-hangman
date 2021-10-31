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

/// Default game mode. Can be changed in the configuration file.
const DEFAULT_REWARDING_SCHEME: RewardingScheme = RewardingScheme::UnhideWhenGuessedChar;

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

/// Delegate the comparison to `Ord`.
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
            // points near the upper left corner are light
            (x as isize * x as isize) + (y as isize * y as isize)
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
            } => (i, DEFAULT_REWARDING_SCHEME),
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

    #[test]
    fn test_image_from() {
        let config: &str = r#"
>o)
(_>   <o)
      (_>
"#;
        let expected: &str = "         \n>o)      \n(_>   <o)\n      (_>\n";
        let image = Image::from(
            &config,
            crate::image::RewardingScheme::UnhideWhenGuessedChar,
        )
        .unwrap();

        assert!(image.visible_points > 0);
        assert_eq!(format!("{}", image), expected);
    }

    #[test]
    fn test_image_yaml_error() {
        let config: &str = "this is no image";
        let image = Image::from_yaml(&config).unwrap_err();
        //println!("{:?}",image);

        assert!(matches!(image, ConfigParseError::NotInYamlFormat(_)));
    }

    /// Test image parsing of configuration file data
    #[test]
    fn test_image_parser_syntax() {
        let config: &str = r#"image: |1
 ab
 c e
 df"#;
        let image = Image::from_yaml(&config);
        //println!("{:?}",image);
        let expected = Ok(Image {
            ichars: [
                ImChar {
                    point: (0, 0),
                    code: 'a',
                },
                ImChar {
                    point: (1, 0),
                    code: 'b',
                },
                ImChar {
                    point: (0, 1),
                    code: 'c',
                },
                ImChar {
                    point: (0, 2),
                    code: 'd',
                },
                ImChar {
                    point: (2, 1),
                    code: 'e',
                },
                ImChar {
                    point: (1, 2),
                    code: 'f',
                },
            ]
            .to_vec(),
            dimension: (3, 3),
            visible_points: 6,
            rewarding_scheme: DEFAULT_REWARDING_SCHEME,
        });

        assert_eq!(image, expected);
    }

    /// Is non image data ignored?
    #[test]
    fn test_image_parser_syntax_ignore() {
        let config: &str = r#"image: |1
 ab
 c
# Comment"#;
        let image = Image::from_yaml(&config).unwrap();
        //println!("{:?}",image);
        let expected = Image {
            ichars: [
                ImChar {
                    point: (0, 0),
                    code: 'a',
                },
                ImChar {
                    point: (1, 0),
                    code: 'b',
                },
                ImChar {
                    point: (0, 1),
                    code: 'c',
                },
            ]
            .to_vec(),
            dimension: (2, 2),
            visible_points: 3,
            rewarding_scheme: DEFAULT_REWARDING_SCHEME,
        };

        assert_eq!(image, expected);
    }

    #[test]
    fn test_image_from_yaml() {
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
    fn test_yaml_image_parser_disclose() {
        //
        // Test yaml.
        let config: &str = "image: |1\n abdef\n c";
        let mut image = Image::from_yaml(&config).unwrap();
        //println!("{:?}",image);
        let expected = Image {
            ichars: [
                ImChar {
                    point: (0, 0),
                    code: 'a',
                },
                ImChar {
                    point: (1, 0),
                    code: 'b',
                },
                ImChar {
                    point: (0, 1),
                    code: 'c',
                },
                ImChar {
                    point: (2, 0),
                    code: 'd',
                },
                ImChar {
                    point: (3, 0),
                    code: 'e',
                },
                ImChar {
                    point: (4, 0),
                    code: 'f',
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
        let image_str = "image: |1\n jensB\n AlisC";
        let image = Image::from_yaml(&image_str).unwrap();
        //println!("{:?}",image);
        let expected = Image {
            ichars: [
                // These are regular image chars.
                ImChar {
                    point: (0, 1),
                    code: 'A',
                },
                ImChar {
                    point: (4, 0),
                    code: 'B',
                },
                ImChar {
                    point: (4, 1),
                    code: 'C',
                },
                // These chars are signature chars.
                ImChar {
                    point: (0, 0),
                    code: 'j',
                },
                ImChar {
                    point: (1, 0),
                    code: 'e',
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
}
