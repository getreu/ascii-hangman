extern crate rand;
mod game;
use game::{Game,GameState};
mod user_interface;
use user_interface::{UserInterface};
mod dictionary;
use dictionary::{Dict};
mod image;
use image::RewardingScheme;

use std::env;
use std::path::PathBuf;
use std::io::prelude::*;
use std::fs::File;
use std::io;
use std::error::Error;


const COMMANDLINE_HELP: &'static str = "\
Hangman is a paper and pencil guessing game for two or more players.  One player
thinks of a word, phrase or sentence and the other tries to guess it by
suggesting letters or numbers, within a certain number of guesses. In this
version for children the computer selects a word, phrase or sentence randomly
out of a word-list defined in a configuration file. In the course of the game
Ascii-Art images - designed for children - are progressively disclosed.

===================================
-----------------------------------
HANGMAN GAME
         ,.
        (_|,.
       ,' /, )_______
    __j o``-'        `
   (\")
    `-j                |
      `-._(           /
         |_\\  |--^.  /
        /_]'|_| /_)_/
           /_]'  /_]'

Lives:  1       Last guess: 3

 g o o _ _ l _ _ k

Type a letter then type [Enter]:
-----------------------------------
===================================

 Usage: hangman [FILE]...
        hangman (-c|--help)
        hangman


`[FILE]` are configuration files containing word-lists and optionally Ascii-Art
images.

When no `[FILE]` argument is given, `[FILE]` defaults to 'hangman-words.txt'. In
case no `[FILE]` is found, a template configuration file 'hangman-words.txt' is
written into the current working directory. Multiple `[FILE]`s are concatted.

`[FILE]` is an Ascii file containing 4 different line-types:

- lines starting with `#` is ignored.

- lines starting with `|` are part of an optional Ascii-Art image shown
  progressively in the course of the game. If not defined here, built in
  Ascii-Art images are used instead.

- lines starting with `:` are game modifier. They change the logic how the image
  is progressively disclosed:
   `:success-rewarding`       Every guessed character shows a bit more of the
                              image. This mode is default.
   `:traditional-rewarding`   Every lost live discloses a bit more of the
                  image. Choose this mode together with a
                              traditional gallows image (not built in).

- lines starting with `-` are _guessing strings_. At the beginning of the game
  one line is randomly chosen and all characters are hidden.  In
  order to give additional hints it is possible to enclose some characters with
  `+*+`.  These words are then displayed in clear. For example a config line:
  `+- Guess *me*+` is shown in the game as: `_ _ _ _ _ _ m e`.

";




const LIVES: u8 = 7;
const PATHSTR: &'static str = "hangman-words.txt";
const OFFSET:  (usize,usize) = (1,1);

const CONF_TEMPLATE: &'static str = "\
### This is a sample word-list for the hangman game

### Sample word-list
#   ----------------
#
# Before every game one line is randomly chosen.
# Empty lines and lines starting with # are ignored.
# Lines with guessing strings must start with '-'.
# Words enclosed with * are not hidden when the game starts:
#   - *guess *me
# shows in the game:
#   g u e s s   _ _
#

- *guess* me
- hang*man*
- good luck


# Lines starting with ':' are game modifier. They change
# the logic how the image is progressively disclosed:
#   ':success-rewarding'       Every guessed character shows a bit more of the
#                              image. This mode is default.
#   ':traditional-rewarding'   Every lost live discloses a bit more of the
#                  image. Choose this mode together with a
#                              traditional gallows image (not built in).


### Sample custom image
#   -------------------
#
# Instead of built in images a word list can use a
# custom image. Lines starting with '|' are interpreted
# as image-lines. Delete '#' in the following lines to
# try out this feature.

#:traditional-rewarding
#|  ______
#|  |    |
#|  |    O
#|  |   /|\\
#|  |    |
#|  |   / \\
#|__|_____
#||      |___
#||_________|
";


const CONF_DEMO: &'static str =
    "- *Demo: add own words to config file and *start* again!*";


// ------------------ MAIN ---------------------------------------------


pub fn read_config(pathstr: &PathBuf) -> Result<String, io::Error> {
        let mut f = try!(File::open(pathstr));
        let mut s = String::new();
        try!(f.read_to_string(&mut s));
        Ok(s)
}


pub fn write_config_template(pathstr: &PathBuf) -> Result<(), io::Error> {
        let mut file = try!(File::create(&pathstr));
        try!(file.write_all( CONF_TEMPLATE.as_bytes() ));
        Ok(())
}





fn main() {
    // SHOW HELP TEXT
    match env::args().nth(1) {
        Some(ref a) if a == "-h" || a == "--help"  => {
            println!("{}", COMMANDLINE_HELP);
            return();
        },
        Some(_) => {},
        None => {},
    };


    // READ CONFIG

    // Read all config files given on command line
    let mut conf_file_paths: Vec<PathBuf> = env::args()
                                .skip(1)
                                .map(|s|PathBuf::from(s))
                                .collect();

    // if no conf_file_paths are given then use default config path
    if conf_file_paths.len() == 0 { conf_file_paths.push(PathBuf::from(PATHSTR)) };

    // read and concat all config files given on command line
    let cwd =  env::current_dir().unwrap();

    let mut config: String = String::new();
    for conf_file_path in &conf_file_paths {

        let path = conf_file_path;
        let c = match read_config(&path) {
            Ok(s)  => s,
            Err(_) => {
                match write_config_template(&path) {
                    Ok(_) => {
                        println!("As no config-file :\n\
                                  \t{:?}\n\
                                  was found a template file is written in the \
                                  current working directory.\n\
                                  \t{:?}\n\n\nPress [Enter] to enter demo mode.",
                                  path,cwd);
                        // wait for [Enter] key
                        let input = &mut String::new();
                        io::stdin().read_line(input).unwrap();
                        CONF_DEMO.to_string()
                    },
                    Err(why) => {
                        println!("Couldn't write hangman template \
                                  config-file:\n\t{:?}\n({})\n\n\
                                  Current working directory is:\n\t{:?}\n\n\
                                  Press [Enter] to enter demo mode.",
                                 path,Error::description(&why),cwd);
                        // wait for [Enter] key
                        let input = &mut String::new();
                        io::stdin().read_line(input).unwrap();
                        CONF_DEMO.to_string()


                    },
                }
            }
        };
        config.push_str(&c);
    }



    // INITIALISE GAME

    let mut ui = UserInterface::new(&config, OFFSET);

    let mut dict = Dict::new(&config);
    if dict.len() == 0 {
         println!("No guessing words in config-file(s):\n\
                   \t{:?}\n\
                   were found. Current working directory is:\n\
                   \t{:?}\n\n\nPress [Enter] to enter demo mode.",
                   conf_file_paths,cwd);
         // wait for [Enter] key
         let input = &mut String::new();
         io::stdin().read_line(input).unwrap();
         dict = Dict::new(CONF_DEMO);
    };



    // PLAY

    'playing: loop {

        let mut game = Game::new(&(dict.get_random_word()), LIVES);
        let chars_to_guess = game.visible_chars();

        // The game loop
        let mut line_buffer = String::new();
        let reader = io::stdin();
        print!("\x1b[2J");  // clear screen
        'running_game: loop {
            if  game.lives > 0 || ui.image.rewarding_scheme ==
                                 RewardingScheme::UnhideWhenLostLife  {
                 ui.image.disclose((game.lives as usize, LIVES as usize),
                      (game.visible_chars(), chars_to_guess)  );
            }
            ui.message= format!("{}",game);
            println!("{}",ui);

            match game.state() {
                GameState::Victory => {
                    println!("Congratulations! You won!");
                    break 'running_game;
                }
                GameState::Defeat => {
                    println!("Sorry, you lost! Better luck next time!");
                    break 'running_game;
                }
                _ => {}
            }

            print!("Type a letter then type [Enter]: ");
            io::stdout().flush().unwrap();

            // Read next char and send it
            line_buffer.clear();
            reader.read_line(&mut line_buffer).unwrap();

            game.guess(
            match line_buffer.chars().next() {
                Some(char_) => {char_}
                None => {continue 'running_game}
            });
        }

        print!("New game? Type [Y]es or [n]o: ");
        io::stdout().flush().unwrap();

        // Read next char
        line_buffer.clear();
        reader.read_line(&mut line_buffer).unwrap();
        println!{"(c) Jens Getreu, 2016."};

        match line_buffer.trim_right().chars().next() {
                Some(char_) if char_ == 'N' || char_ == 'n'=> {break 'playing}
                Some(_) => {}
                None => {}
        }
    }
}

