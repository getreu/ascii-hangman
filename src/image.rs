//! Holds a dictionary of built-in ASCII art images and manages the piecemeal disclosure to the
//! image.  Also parses user provided images if given in the configuration file.

extern crate rand;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::cmp::{Ord, Ordering};
use std::fmt;
extern crate crossterm;
use crate::Render;
use crossterm::cursor;

/// Identifier tagging image data in configuration files.
pub const CONF_LINE_IDENTIFIER__IMAGE: char = '|';

/// Threshold to decide from how many characters on the images is considered to be "big".
/// Big images are disclosed with another algorithm.
const BIG_IMAGE: usize = 100; // sort algorithm <-> random algorithm

/// A collection of built-in images from whom one is chosen at the start of the game.
// first char of image lines must be '|'
const DEFAULT_IMAGES: &[&str] = &[
    r#"
|    ,,,,,
|   (o   o)
|    /. .\
|   (_____)
|     : :
|    ##O##'
|  ,,,: :,,,
| _)\ : : /(____
|{  \     /  ___}
| \/)     ((/
|  (_______)
|    :   :
|    :   :
|   / \ / \
|   """ """
"#,
    r#"
|    |\_|X|_/|
|   /         \
| =(  O     O  )=
|  -\    o    /-
|   / .-----. \
| /_ | o   o |_ \
|(U  |       |  U)
|   _|_     _|_
|  (   )---(   )
"#,
    r#"
|        _.---._    /\\
|     ./'       "--`\//
|   ./              o \
|  /./\  )______   \__ \
| ./  / /\ \   | \ \  \ \
|    / /  \ \  | |\ \  \7
|     "     "    "  "        VK
"#,
    r#"
|       ,.
|      (_|,.
|     ,' /, )_______   _
|  __j o``-'        `.'-)'
| (")                 \'
|  `-j                |
|    `-._(           /
|       |_\  |--^.  /
|      /_]'|_| /_)_/
|         /_]'  /_]'
# Author: hjw
"#,
    r#"
|        _
|       [ ]
|      (   )
|       |>|
|    __/===\__
|   //| o=o |\\
| <]  | o=o |  [>
|     \=====/
|    / / | \ \
|   <_________>
"#,
    r#"
|                          (_)(_)
|                          /     \
|                         /       |
|                        /   \  * |
|          ________     /    /\__/
|  _      /        \   /    /
| / \    /  ____    \_/    /
|//\ \  /  /    \         /
|V  \ \/  /      \       /
|    \___/        \_____/
"#,
    r#"
|         .-.
|        (. .)__,')
|        / V      )
|  ()    \  (   \/
|<)-`\()  `._`._ \
|  <).>=====<<==`'====
|   C-'`(>
# Author: hjw
"#,
    r#"
| >(. )
|  |  (     /)
|  |   \___/ )
|  (   ----- )  >@)_//   >@)_//  >@)_//  >@)_//
|   \_______/    (__)     (__)    (__)    (__)
|~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~
"#,
    r#"
|           __
|           /(`o
|     ,-,  //  \\
|    (,,,) ||   V
|   (,,,,)\//
|   (,,,/w)-'
|   \,,/w)
|   `V/uu
|     / |
|     | |
|     o o
|     \ |
|\,/  ,\|,.  \,/
"#,
    r#"
|o
| \_/\o
|( Oo)                    \|/
|(_=-)  .===O-  ~~Z~A~P~~ -O-
|/   \_/U'                /|\
|||  |_/
|\\  |
|{K ||
| | PP
| | ||
| (__\\
# Author: ac
"#,
    r#"
|      ______
|     /     /\
|    /     /  \
|   /_____/----\_    (
|  "     "          ).
| _ ___          o (:') o
|(@))_))        o ~/~~\~ o
|                o  o  o
"#,
    r#"
|                             _______     |\
|                            |License|    | \
|  _____                     | ~~*~~ |    |  \
| |     |  (((        .--.   |_______|    |
| |DrJRO| ~OvO~ __   (////)               |
| |     | ( _ )|==|   \__/                |
| |o    |  \_/ |_(|  /    \   _______     |
| |     | //|\\   \\//|  |\\  |__o__|     |
| |   __|//\_/\\ __\/ |__|//  |__o__|     |
| |  |==""//=\\""====|||||)   |__o__|     |
|_|__||_|_||_||_____||||||____|__o__|_____|
|    ||  (_) (_)    ||||||                \
|    []             [(_)(_)
"#,
    r#"
|   _     _ 
|  ( |_ _| ) 
|   ( .". )   
|  _( (Y) )_  
| / /,---.\ \ 
|/ / | + | \ \
|\_)-"   "-(_/  
|  |_______| 
|  _)  |  (_ 
| (___,'.___)  hjw
# Art by Hayley Jane Wakenshaw
# (slightly modified)
"#,
    r#"
|          |
|        \ _ /
|      -= (_) =-
|        /   \         _\/_
|          |           //o\  _\/_
|   _____ _ __ __ ____ _ | __/o\\ _
| =-=-_-__=_-= _=_=-=_,-'|"'""-|-,_
|  =- _=-=- -_=-=_,-"          |
|jgs =- =- -=.--"
# Art by Genoveva Galarza
"#,
    r#"
|        __I__
|   .-'"  .  "'-.
| .'  / . ' . \  '.
|/_.-..-..-..-..-._\ .---------------------------------.
|         #  _,,_   ( I hear it might rain people today )
|         #/`    `\ /'---------------------------------'
|         / / 6 6\ \
|         \/\  Y /\/       /\-/\
|         #/ `'U` \       /a a  \               _
|       , (  \   | \     =\ Y  =/-~~~~~~-,_____/ )
|       |\|\_/#  \_/       '^--'          ______/
|       \/'.  \  /'\         \           /
|        \    /=\  /         ||  |---'\  \
|   jgs  /____)/____)       (_(__|   ((__|
# Art by Joan Stark
"#,
    r#"
| [][][] /""\ [][][]
|  |::| /____\ |::|
|  |[]|_|::::|_|[]|
|  |::::::__::::::|
|  |:::::/||\:::::|
|  |:#:::||||::#::|
| #%*###&*##&*&#*&##
|##%%*####*%%%###*%*#
"#,
    r#"
|  ,-~~-.___.
| / |  '     \         
|(  )         0  
| \_/-, ,----'            
|    ====           // 
|   /  \-'~;    /~~~(O)
|  /  __/~|   /       |     
|=(  _____| (_________|
"#,
    r#"
|  \,`/ / 
| _)..  `_
|( __  -\
|    '`.                  
|   ( \>_-_,   
|   _||_ ~-/    W<
"#,
    r#"
|            __:.__
|           (_:..'"=
|            ::/ o o\         AHAH!
|           ;'-'   (_)     Spaceman Spiff      .
|           '-._  ;-'        wins again !  _'._|\/:
|           .:;  ;                .         '- '   /_
|          :.. ; ;,                \       _/,    "_<
|         :.|..| ;:                 \__   '._____  _)
|         :.|.'| ||                            _/ /
|snd      :.|..| :'                           `;--:
|         '.|..|:':       _               _ _ :|_\:
|      .. _:|__| '.\.''..' ) ___________ ( )_):|_|:
|:....::''::/  | : :|''| "/ /_=_=_=_=_=/ :_[__'_\3_)
| ''''      '-''-'-'.__)-'
# Art by Shanaka Dias
"#,
    r#"
|  _,                          _                
|.'  `.                  ___.>"''-..-.          
|`-.   ;           .--"""        .-._@;         
|   ;  !_.--..._ .'      /     .[_@'`'.         
|  ;            /       : .'  ; :_.._  `.       
|  :           ;        ;[   _T-"  `.'-. `-.    
|   \        .-:      ; `.`-=_,88p.   _.}.-"    
|    `-.__.-'   \    /L._ Y",P$T888;  ""        
|             .-'_.-'  / ;$$$$$$]8P;            
|             \ /     / / "Y$$P" ^"             
|     fsc      ;\_    `.\_._                    
|              ]__\     \___;
"#,
    r#"
|        _
|      _<_/_
|   __/    _>
|  '\  '  |
|    \___/
|    /+++\
| o=|..|..|
|   | o/..|
|0==|+++++|
| 0======/
"#,
    r#"
|        _../|_
|      ='__   _~-.
|           \'  ~-`\._
|                 |/~`
|   .    .    .    .    .
|_.`(._.`(._.`(._.`(._.`(._
"#,
    r#"
|                        ____
|                   .---'-    \
|      .-----------/           \
|     /           (         ^  |   __
|&   (             \        O  /  / .'
|'._/(              '-'  (.   (_.' /
|     \                    \     ./
|      |    |       |    |/ '._.'
|       )   @).____\|  @ |
|   .  /    /       (    | mrf
|  \|, '_:::\  . ..  '_:::\ ..\).
# Art by Morfina
"#,
    r#"
|           __n__n__
|    .------`-\00/-'
|   /  ##  ## (oo)
|  / \## __   ./
|     |//YY \|/
|snd  |||   |||
# Art by Shanaka Dias
"#,
    r#"
|                       .-'~~~-.
|                     .'o  oOOOo`.
|                    :~~~-.oOo   o`.
|                     `. \ ~-.  oOOo.
|                       `.; / ~.  OO:
|                       .'  ;-- `.o.'
|                      ,'  ; ~~--'~
|                      ;  ;
|_______\|/__________\\;_\\//___\|/________
"#,
    r#"
|   (__  '.
|    /_____)
|   ()@ @ )))
|    'C ,()(()
|    ,.'_'.' \
| __/ )   (--'
|'._./     \
|   (_._._._)
|    _|| ||_
|mrf(__.).__)
"#,
    r#"
|        o    .   _     .
|          .     (_)         o
|   o      ____            _       o
|  _   ,-/   /)))  .   o  (_)   .
| (_)  \_\  ( e(     O             _
| o       \/' _/   ,_ ,  o   o    (_)
|  . O    _/ (_   / _/      .  ,        o
|     o8o/    \\_/ / ,-.  ,oO8/( -TT
|    o8o8O | } }  / /   \Oo8OOo8Oo||     O
|   Oo(""o8"""""""""""""""8oo""""""")
|  _   `\`'                  `'   /'   o
| (_)    \                       /    _   .
|      O  \           _         /    (_)
|o   .     `-. .----<(o)_--. .-'
|   --------(_/------(_<_/--\_)--------hjw
"#,
    r#"
|                \||/
|                |  @___oo
|      /\  /\   / (__,,,,|
|     ) /^\) ^\/ _)
|     )   /^\/   _)
|     )   _ /  / _)
| /\  )/\/ ||  | )_)
|<  >      |(,,) )__)
| ||      /    \)___)\
| | \____(      )___) )___
|  \______(_______;;; __;;;
"#,
    r#"
|   (\{\
|   { { \ ,~,
|  { {   \)))  *
|   { {  (((  /
|    {/{/; ,\/
|       (( '
|        \` \
|        (/  \
|ejm     `)  `\
"#,
    r#"
|                    /
|               ,.. /
|             ,'   ';
|  ,,.__    _,' /';  .
| :','  ~~~~    '. '~
|:' (   )         )::,
|'. '. .=----=..-~  .;'
| '  ;'  ::   ':.  '"
|   (:   ':    ;)
|    \\   '"  ./
|     '"      '"
# DR J
"#,
    r#"
|     __/\__
|. _  \\''//
|-( )-/_||_\
| .'. \_()_/
|  |   | . \
|  |mrf| .  \
| .'. ,\_____'.
"#,
    r#"
|         _.-.
|       ,'/ //\
|      /// // /)
|     /// // //|
|    /// // ///
|   /// // ///
|  (`: // ///
|   `;`: ///
|   / /:`:/
|  / /  `'
| / /
|(_/  hh
"#,
    r#"
| _____
||A .  | _____
|| /.\ ||A ^  | _____
||(_._)|| / \ ||A _  | _____
||  |  || \ / || ( ) ||A_ _ |
||____V||  .  ||(_'_)||( v )|
|       |____V||  |  || \ / |
|              |____V||  .  |
|                     |____V| ejm98
"#,
    r#"
|      !!!!\\\\
|    '`!_  ||||
|     ` \`-'''|
|       `\   /
|        )\  \
| ejm   /  \  \
|           \|
"#,
];

/// One character of the ASCII art image.
#[derive(PartialOrd, Eq, PartialEq, Debug, Copy, Clone)] //omitting Ord
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
        weight(&self).cmp(&weight(&other))
    }
}

#[derive(Debug)]
/// An ASCII-art image.
pub struct Image {
    pub ichars: Vec<ImChar>,
    pub offset: (usize, usize),
    pub dimension: (u8, u8),
    pub visible_points: usize,
}

impl Render for Image {
    /// Renders and prints the image on the screen. It would be more consistent to implement Display
    /// for Image, but crossterm does not supprt `print!(f, ...)`. Therefor, it is not on option
    /// here.
    fn render(&self) {
        use std::io;
        use std::io::prelude::*;

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

            print!("{}", &code);

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
    /// Constructor reading image data from configuration files.
    pub fn new(string: &str, offset: (usize, usize)) -> Self {
        let mut v: Vec<ImChar> = Vec::new();

        for (y, line) in string
            // split in lines
            .lines()
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
            }
        }
    }

    /// Sets how much of the image will disclosed next time the image is rendered.
    pub fn disclose(&mut self, frac: (usize, usize)) {
        let l = self.ichars.len();

        let as_points = |(n, d)| (5 * l * (d - n) as usize / d as usize + l) / 6;

        if frac.1 > 0 {
            self.visible_points = as_points(frac);
        };
    }
}
