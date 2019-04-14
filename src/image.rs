extern crate rand;
use rand::Rng;
use std::cmp::{Ord, Ordering};
use std::fmt;

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
const DEFAULT_IMAGES: & [&str] = &[
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
pub struct ImageChar {
    pub point: (u8, u8),
    pub char_: char,
}

impl fmt::Display for ImageChar {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.char_)
    }
}

// Ord enables us to v.sort()
impl Ord for ImageChar {
    fn cmp(&self, other: &Self) -> Ordering {
        fn weight(ic: &ImageChar) -> isize {
            let &ImageChar {
                point: (x, y),
                ..
            } = ic;
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
    pub ichars: Vec<ImageChar>,
    pub offset: (usize, usize),
    pub dimension: (u8, u8),
    pub visible_points: usize,
    pub rewarding_scheme: RewardingScheme,
}

impl fmt::Display for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut s = String::new();
        for ic in self.ichars.iter().take(self.visible_points) {
            let &ImageChar {
                point: (x, y),
                ..
            } = ic;
            s = s
                + "\x1b["
                + &(y as usize + 1 + self.offset.1).to_string()
                + ";"
                + &(x as usize + 1 + self.offset.0).to_string()
                + "f"
                + &c.to_string();
        }
        // after printing the image s, bring the cursor below
        write!(
            f,
            "{}\x1b[{};0f",
            s,
            (self.dimension.1 as usize + 1 + self.offset.1).to_string()
        )
    }
}

impl Image {
    pub fn new(string: &str, offset: (usize, usize)) -> Image {
        let mut v: Vec<ImageChar> = Vec::new();

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
                .map(|(x, c)| ImageChar {
                    point: (x as u8, y as u8),
                    char_: c,
                })
                .collect();
            v.append(&mut ii);
        }

        // find dimensions
        let mut x_max = 0;
        let mut y_max = 0;
        for i in v.iter() {
            let &ImageChar {
                point: (x, y),
                ..
            } = i;
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
            rand::thread_rng().shuffle(&mut v); // points appear randomly.
        }

        if v.len() == 0 {
            Image::new(rand::thread_rng().choose(&DEFAULT_IMAGES).unwrap(), offset)
        } else {
            Image {
                ichars: v,
                offset: offset,
                dimension: (x_max,y_max),
                visible_points: v_len,
                rewarding_scheme: rewarding_scheme,
            }
        }
    }

    pub fn disclose(&mut self, lives_frac: (usize, usize), guessed_chars_frac: (usize, usize)) {
        let l = self.ichars.len();

        let as_points = |(n, d)| (3 * l * (d - n) as usize / d as usize + 1 * l) / 4;

        self.visible_points = match self.rewarding_scheme {
            RewardingScheme::UnhideWhenGuessedChar => as_points(guessed_chars_frac),
            RewardingScheme::UnhideWhenLostLife => as_points(lives_frac),
        };
    }
}
