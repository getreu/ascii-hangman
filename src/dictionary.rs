#![allow(clippy::filter_map)]
extern crate rand;
use crate::image::CONF_LINE_IDENTIFIER__IMAGE;
use rand::thread_rng;
use rand::seq::SliceRandom;



// Config file syntax error message
pub const CONF_SYNTAX_ERROR: &str = "

SYNTAX ERROR in config file!
Every line has to start with one of the following characters:
'#' (comment line), '-' (guessing string), '|' (ASCII-Art image) or ':' (game modifier).
Edit config file and start again.\n";

// comments in config file start with
pub const CONF_LINE_IDENTIFIER__COMMENT: char = '#';

// comands in config-file start with
pub const CONF_LINE_IDENTIFIER__CONTROL: char = ':';

// guessing strings in config-file start with
pub const CONF_LINE_IDENTIFIER__WORD: char = '-';

// a modifier tagging parts of the string to be visible from the start, e.g.
// "guess_-me_: will be shown as "_ _ _ _ _ - m e"
pub const CONF_LINE_WORD_MODIFIER__VISIBLE: char = '_';

#[derive(Debug)]
pub struct Dict {
    wordlist: Vec<String>,
}

impl Dict {
    pub fn len(&self) -> usize {
        self.wordlist.len()
    }
    pub fn new(lines: &str) -> Self {
        Self{wordlist :
          // remove Unicode BOM if present (\u{feff} has in UTF8 3 bytes).
          if lines.starts_with('\u{feff}') { &lines[3..] } else { &lines[..] }
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
                             panic!("{}\nError in line: {}: \"{}\"\n\n",
                                     CONF_SYNTAX_ERROR, n+1, l)
                     })
            .collect()
        }
    }

    pub fn get_random_word(&self) -> String {
        let mut rng = thread_rng();
        (&self.wordlist).choose(&mut rng).unwrap().to_string()
    }
}
