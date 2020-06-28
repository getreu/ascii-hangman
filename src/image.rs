//! Holds a dictionary of built-in ASCII art images and manages the piecemeal disclosure to the
//! image.  Also parses user provided images if given in the configuration file.

extern crate rand;
use crate::application::LIVES;
use crate::dictionary::ConfigParseError;
use crate::game::Game;
use rand::seq::SliceRandom;
use rand::thread_rng;
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
|    ____
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
    r#"
|  ,~~--~~-.
| +      | |\
| || |~ |`,/-\
| *\_) \_) `-'#,
"#,
    r#"
|  (.  \
|   \  |
|    \ |___(\--/)
|  __/    (  . . )
| "'._.    '-.O.'
|      '-.  \ "|\
|         '.,,/'.,,mrf
"#,
    r#"
|             __
|   ,'```--'''  ``-''-.
| ,'            ,-- ,-'.
|(//            `"'| 'a \
|  |    `;         |--._/
|  \    _;-._,    /
|   \__/\\   \__,'
|    ||  `'   \|\\
|    \\        \\`'
|hjw  `'        `'
"#,
    r#"
|\\             //
| \\\' ,      / //
|  \\\//,   _/ //,
|   \_-//' /  //<,
|     \ ///  <//`
|    /  >>  \\\`__/_
|   /,)-^>> _\` \\\
|   (/   \\ //\\
|       // _//\\\\
|      ((` ((
"#,
    r#"
|>o)
|(_>   <o)
|      (_>
"#,
    r#"
|              I~
|          I~ /V\  I~
|      I~ /V\ | | /V\  I~
| @ @ /V\ | |_|_|_| | /V\ @ @
|@@@@@| |_| |_/V\_| |_| |@@@@@
|@@@@@| | |_|_|_|_|_| | |@@@@@
|@@@@@|_|_V_V|   |V_V_|_|@@@@@
|_._._._._._._._._._._._._._._
|:::::::::::::|X|:::::::::::::
|Sher^
"#,
    r#"
| W                   __
|[ ]                 |::|
| E          ._.     |::|   ._.
| |\         |:| ._. |::|   |/|
| \ \\|/     |:|_|/| |::|_  |/|
|  |-( )-    |:|"|/|_|::|\|_|/| _
|  | V L     |:|"|/|||::|\|||/||:|
|  \    `  ___   ~~~~~~~~~~~~~~~~~~~~
|   |    \/  /     ~~~~ ~~~~ ~~~ ~~~
"#,
    r#"
|      .___.
|     /     \
|    | O _ O |
|    /  \_/  \
|  .' /     \ `.
| / _|       |_ \
|(_/ |       | \_)
|    \       /
|   __\_>-<_/__
|   ~;/     \;~
"#,
    r#"
|    ,--.
|    \  _\_
|    _\/_|_\____.'\
|  -(___.--._____(
|       \   \
|        \   \
|         `--'
|  jg
"#,
    r#"
|       __________________________
|      /   | |______| |___     __/
|     /  , | |  /\  | | ^ |   |       ,--.
|   ,' ,'| | |.'  `.| |/ \|   |      /    \
| ,' ,'__| | |______| |___|   |      \    /
|/         |          |   |   |     _ `--'
|[   ,--.  |          |,--|   |]   (_)
||__/    \_|__________/    \__|= o
|   \    /            \    /
|bmw `--'              `--'
"#,
    r#"
|   _..__.          .__.._
|  .^"-.._ '-(\__/)-' _..-"^.
|         '-.' oo '.-'
|            `-..-'       fsc
"#,
    r#"
|                          .
|                      _.:/ )
|    _              .-Q      `._
| '\(o7/'         o(.__         '-.
| `.( ).'           `_/    )
|    H       ._      '-._.'         kOs
|    w       ( \         /
|             \ '.     .'   )
"#,
    r#"
|   .
|  / \__        .. _
|  \.'  '._o    \_|_) ))
|__(  __ / /      ).
|\  _( ,/ /.____.' /
| '' '..-'        |
|        \    _   (
|         )v /-'._ )
|        ////   |//
|       // \\   //
|      //   \\ ||\\
|   --"------"-"--"--  mrf
"#,
    r#"
|   a'!   _,,_   a'!   _,,_     a'!   _,,_
|     \\_/    \    \\_/    \      \\_/    \.-,
|      \, /-( /'-,  \, /-( /'-,    \, /-( /
|      //\ //\\     //\ //\\       //\ //\\
|jrei
"#,
    r#"
|      ,__,
| (/__/\oo/\__(/
|   _/\/__\/\_
|    _/    \_    b'ger
"#,
    r#"
|          (    )
|           (oo)
|  )\.-----/(O O)
| # ;       / u
|   (  .   |} )
|    |/ `.;|/;
|    "     " "
#unknown
"#,
    r#"
|                __
|               /\/'-,
|       ,--'''''   /"
| ____,'.  )       \___
|'"""""------'"""`-----'
#pb
"#,
    r#"
|          _
|   ______/ \-.   _         _ __ _         _    _
|.-/     (    o\_//        / |..| \       / >--< \
| |  ___  \_/\---'         \/ || \/       \|  \ |/
| |_||  |_||       wtx      |_''_|         |_||_|
"#,
    r#"
|                    _J""-.
|       .-""L_      /o )   \ ,';
|  ;`, /   ( o\     \ ,'    ;  /
|  \  ;    `, /      "-.__.'"\_;
|  ;_/"`.__.-"
|                              fsc
"#,
    r#"
|       >=<                >=<
|  ,.--'  ''-.        ,.--'  ''-.
|  (  )  ',_.'        (  )  ',_.'
|   Xx'xX              mn'mn`
|                             Asik
"#,
    r#"
|        .-' '-.
|       /       \
|      |,-,-,-,-,|
|     ___   |
|    _)_(_  |
|    (/ \)  |
|    _\_/_  /)
|   / \_/ \//
|   |(   )\/
|   ||)_(
|   |/   \
|   n|   |
|  / \   |
|  |_|___|
|     \|/
|jgs _/L\_
"#,
    r#"
|  oo`'._..---.___..-
| (_,-.        ,..'`
|      `'.    ;
|         : :`
|        _;_;      jrei
"#,
    r#"
|       `.oo'    |    `oo.'
|    ,.  (`-'    |    `-')  ,.
|   '^\`-' )     |     ( `-'/^`
|      c-L'-     |     -`_-)
|-bf-
"#,
    r#"
|       .___,
|    ___('v')___
|    `"-\._./-"'
|hjm     ^ ^
"#,
    r#"
| ;-.               ,
|  \ '.           .'/
|   \  \ .---. .-' /
|    '. '     `\_.'
|      |(),()  |     ,
|      (  __   /   .' \
|     .''.___.'--,/\_,|
|    {  /     \   }   |
|     '.\     /_.'    /
|      |'-.-',  `; _.'
|  jgs |  |  |   |`
|      `""`""`"""`
"#,
    r#"
|           __,---.__
|        ,-'         `-.__
|      &/           `._\ _\
|      /               ''._
|      |   ,             (")
| jrei |__,'`-..--|__|--''
"#,
    r#"
|    ;     /        ,--.
|   ["]   ["]  ,<  |__**|
|  /[_]\  [~]\/    |//  |
|   ] [   OOO      /o|__|   Phs
"#,
    r#"
|       |
|  m1a  |
|       |
|   /   |   \
|   \   |   /
| .  --\|/--  ,
|  '--|___|--'
|  ,--|___|--,
| '  /\o o/\  `
|   +   +   +
|    `     '
"#,
    r#"
|        ___`\`,`.
|   ()    `-.--. |.
|    \`-.-(e(e.' .1
|     ;.___..~   _")
|    <./|_|`-~.   |
|              \  |
|               | |
|               | |
|     (PS)      | /\
"#,
    r#"
|       O>         _
|      ,/)          )_
|  -----<---<<<   )   )
|       ``      ` _)
|                jrei
"#,
    r#"
|    _|\_
|     ("}
|  i_.-@-._ _
|  8--,  .-`*
|  I  /==\
|  I  |   \
|  I  /___\
|snd
"#,
    r#"
| ,__ ., __, ,,,,
| '--/,,\--'\*\%\*\
|   //  \\\'\'%.\'%\
|    '..'//'%\.\%/\\'.^
|       \\'/'/%''/\'
|         ||     ||
|         "      "
#morfina
"#,
];

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
    /// Constructor reading image data from configuration files.
    pub fn new(string: &str) -> Result<Self, ConfigParseError> {
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

        if file_syntax_test1.is_err() {
            return Err(file_syntax_test1.unwrap_err());
        };

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
            let mut rng = thread_rng();
            // this is recursive!
            Self::new((&DEFAULT_IMAGES).choose(&mut rng).unwrap())
        } else {
            Ok(Self {
                ichars: v,
                dimension,
                visible_points: v_len,
                rewarding_scheme,
            })
        }
    }

    /// Discloses parts of the image according to the course of the play.
    pub fn update(&mut self, game: &Game) {
        match self.rewarding_scheme {
            RewardingScheme::UnhideWhenGuessedChar => {
                self.hide((game.secret.hidden_chars(), game.secret.chars_to_guess));
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
    use super::*;

    /// Test image parsing of configuration file data
    #[test]
    fn test_image_parser_syntax() {
        let config: &str = r#"
|ab
|cd"#;
        let image = Image::new(&config);
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
        let image = Image::new(&config).unwrap();
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
        let image = Image::new(&config).unwrap();

        assert!(image.visible_points > 0);
        assert_eq!(format!("{}", image), expected);
    }

    #[test]
    fn test_image_parser_built_in_image() {
        let config: &str = "this is no image";
        let image = Image::new(&config).unwrap();
        //println!("{:?}",image);

        assert!(image.visible_points > 0);
    }

    /// disclose image progressively
    #[test]
    fn test_image_parser_disclose() {
        let config: &str = "|abcde";
        let mut image = Image::new(&config).unwrap();
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
            dimension: (5, 1),
            visible_points: 5,
            rewarding_scheme: DEFAULT_REWARDING_SCHEME,
        };
        assert!(image == expected);

        image.hide((5, 5));
        assert!(image.visible_points == 0);

        image.hide((1, 5));
        assert!(image.visible_points == 4);

        image.hide((0, 5));
        assert!(image.visible_points == 5);
    }

    /// indent of game modifier is not allowed

    /// test game modifier spelling
    #[test]
    fn test_image_parser_error_misspelled() {
        let config = "\n\n:traditional-rewardXing";
        let dict = Image::new(&config);
        let expected = Err(ConfigParseError::GameModifier {
            line_number: 3,
            line: ":traditional-rewardXing".to_string(),
        });
        assert_eq!(dict, expected);
    }
}
