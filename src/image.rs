extern crate rand;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::cmp::{Ord, Ordering};
use std::fmt;
extern crate crossterm;
use crate::Render;
use crossterm::cursor;

// comands in config-file start with
pub const CONF_LINE_IDENTIFIER__CONTROL: char = ':';

// the default can be changed by one of the following switches
const DEFAULT_REWARDING_SCHEME: RewardingScheme = RewardingScheme::UnhideWhenGuessedChar;

// Keyword to switch rewarding scheme
// :traditional-rewarding
const UNHIDE_WHEN_LOST_LIVE_IDENTIFIER: &str = "traditional-rewarding";

// Keyword to switch rewarding scheme
// :success-rewarding
const UNHIDE_WHEN_GUESSED_CHAR_IDENTIFIER: &str = "success-rewarding";

// images in config file start with
pub const CONF_LINE_IDENTIFIER__IMAGE: char = '|';

const BIG_IMAGE: usize = 100; // sort algorithm <-> random algorithm

// first char of image lines must be '|'
const DEFAULT_IMAGES: &[&str] = &[
    "
|    ,,,,,
|   (o   o)
|    /. .\\
|   (_____)
|     : :
|    ##O##'
|  ,,,: :,,,
| _)\\ : : /(____
|{  \\     /  ___}
| \\/)     ((/
|  (_______)
|    :   :
|    :   :
|   / \\ / \\
|   \"\"\" \"\"\"
",
    "
|    |\\_|X|_/|
|   /         \\
| =(  O     O  )=
|  -\\    o    /-
|   / .-----. \\
| /_ | o   o |_ \\
|(U  |       |  U)
|   _|_     _|_
|  (   )---(   )
",
    "
|        _.---._    /\\\\
|     ./'       \"--`\\//
|   ./              o \\
|  /./\\  )______   \\__ \\
| ./  / /\\ \\   | \\ \\  \\ \\
|    / /  \\ \\  | |\\ \\  \\7
|     \"     \"    \"  \"        VK
",
    "
|       ,.
|      (_|,.
|     ,' /, )_______   _
|  __j o``-'        `.'-)'
| (\")                 \\'
|  `-j                |
|    `-._(           /
|       |_\\  |--^.  /
|      /_]'|_| /_)_/
|         /_]'  /_]'
# Author: hjw
",
    "
|        _
|       [ ]
|      (   )
|       |>|
|    __/===\\__
|   //| o=o |\\\\
| <]  | o=o |  [>
|     \\=====/
|    / / | \\ \\
|   <_________>
",
    "
|                          (_)(_)
|                          /     \\
|                         /       |
|                        /   \\  * |
|          ________     /    /\\__/
|  _      /        \\   /    /
| / \\    /  ____    \\_/    /
|//\\ \\  /  /    \\         /
|V  \\ \\/  /      \\       /
|    \\___/        \\_____/
",
    "
|         .-.
|        (. .)__,')
|        / V      )
|  ()    \\  (   \\/
|<)-`\\()  `._`._ \\
|  <).>=====<<==`'====
|   C-'`(>
# Author: hjw
",
    "
| >(. )
|  |  (     /)
|  |   \\___/ )
|  (   ----- )  >@)_//   >@)_//  >@)_//  >@)_//
|   \\_______/    (__)     (__)    (__)    (__)
|~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~
",
    "
|           __
|           /(`o
|     ,-,  //  \\\\
|    (,,,) ||   V
|   (,,,,)\\//
|   (,,,/w)-'
|   \\,,/w)
|   `V/uu
|     / |
|     | |
|     o o
|     \\ |
|\\,/  ,\\|,.  \\,/
",
    "
|o
| \\_/\\o
|( Oo)                    \\|/
|(_=-)  .===O-  ~~Z~A~P~~ -O-
|/   \\_/U'                /|\\
|||  |_/
|\\\\  |
|{K ||
| | PP
| | ||
| (__\\\\
# Author: ac
",
    "
|      ______
|     /     /\\
|    /     /  \\
|   /_____/----\\_    (
|  \"     \"          ).
| _ ___          o (:') o
|(@))_))        o ~/~~\\~ o
|                o  o  o
",
    "
|                             _______     |\\
|                            |License|    | \\
|  _____                     | ~~*~~ |    |  \\
| |     |  (((        .--.   |_______|    |
| |DrJRO| ~OvO~ __   (////)               |
| |     | ( _ )|==|   \\__/                |
| |o    |  \\_/ |_(|  /    \\   _______     |
| |     | //|\\\\   \\\\//|  |\\\\  |__o__|     |
| |   __|//\\_/\\\\ __\\/ |__|//  |__o__|     |
| |  |==\"\"//=\\\\\"\"====|||||)   |__o__|     |
|_|__||_|_||_||_____||||||____|__o__|_____|
|    ||  (_) (_)    ||||||                \\
|    []             [(_)(_)
",
];

#[derive(PartialOrd, Eq, PartialEq, Debug, Copy, Clone)] //omitting Ord
pub struct ImChar {
    pub point: (u8, u8),
    pub code: char,
}

impl fmt::Display for ImChar {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.code)
    }
}

// Ord enables us to v.sort()
impl Ord for ImChar {
    fn cmp(&self, other: &Self) -> Ordering {
        fn weight(ic: &ImChar) -> isize {
            let &ImChar { point: (x, y), .. } = ic;
            // points near the lower left corner are light
            x as isize - y as isize
        }
        weight(&self).cmp(&weight(&other))
    }
}

#[derive(Debug, PartialEq)]
pub enum RewardingScheme {
    UnhideWhenLostLife,
    UnhideWhenGuessedChar,
}

#[derive(Debug)]
pub struct Image {
    pub ichars: Vec<ImChar>,
    pub offset: (usize, usize),
    pub dimension: (u8, u8),
    pub visible_points: usize,
    pub rewarding_scheme: RewardingScheme,
}

impl Render for Image {
    fn render(&self) {
        use std::io::prelude::*;                                                           
        use std::io;   
        
        let cursor = cursor();
        for ic in self.ichars.iter().take(self.visible_points) {
            let &ImChar {
                point: (x, y),
                code,
            } = ic;
            cursor
                .goto(
                    (self.offset.1 + (x as usize) + 1) as u16,
                    (self.offset.0 + (y as usize) + 1) as u16,
                )
                .expect("Can not set cursor position.");

            print!("{}", &code );

            // The following flush() is necessary on Windows terminals that do not understand ANSI
            // escape code such as Window 7, 8 and older 10. BTW, in 2016, Microsoft released the
            // Windows 10 Version 1511 update which unexpectedly implemented support for ANSI
            // escape sequences.  
            // [ANSI escape code](https://en.wikipedia.org/wiki/ANSI_escape_code#Windows)
           
            io::stdout().flush().ok().expect("Could not flush stdout");
        }
        // after printing the image s, bring the cursor below
        cursor
            .goto(0, (self.dimension.1 as usize + 1 + self.offset.1) as u16)
            .expect("Can not move cursor.");
    }
}

impl Image {
    pub fn new(string: &str, offset: (usize, usize)) -> Self {
        let mut v: Vec<ImChar> = Vec::new();

        let mut rewarding_scheme: RewardingScheme = DEFAULT_REWARDING_SCHEME;
        for (y, line) in string
            // split in lines
            .lines()
            // interpret identifier line
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
                // save in ImageChar object
                .map(|(x, c)| ImChar {
                    point: (x as u8, y as u8),
                    code: c,
                })
                .collect();
            v.append(&mut ii);
        }

        // find dimensions
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

        // order points
        let v_len = v.len();
        if v_len <= BIG_IMAGE {
            v.sort(); // Sort algorithm, see "impl Ord for ImageChar"
        } else {
            let mut rng = thread_rng();
            (&mut v).shuffle(&mut rng); // points appear randomly.
        }

        if v.is_empty() {
            let mut rng = thread_rng();
            Self::new((&DEFAULT_IMAGES).choose(&mut rng).unwrap(), offset)
        } else {
            Self {
                ichars: v,
                offset,
                dimension: (x_max, y_max),
                visible_points: v_len,
                rewarding_scheme,
            }
        }
    }

    pub fn disclose(&mut self, lives_frac: (usize, usize), guessed_chars_frac: (usize, usize)) {
        let l = self.ichars.len();

        let as_points = |(n, d)| (3 * l * (d - n) as usize / d as usize + l) / 4;

        self.visible_points = match self.rewarding_scheme {
            RewardingScheme::UnhideWhenGuessedChar => as_points(guessed_chars_frac),
            RewardingScheme::UnhideWhenLostLife => as_points(lives_frac),
        };
    }
}
