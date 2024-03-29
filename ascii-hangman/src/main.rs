//! Loads the configuration and runs the game workflow.
//! This file not sourced for the wasm32 (wasm) target.
#![cfg(not(target_arch = "wasm32"))]

use ascii_hangman_backend::game::State;
use ascii_hangman_backend::Backend;
use ascii_hangman_backend::HangmanBackend;
use ascii_hangman_backend::{AUTHOR, CONF_TEMPLATE, TITLE, VERSION};
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process;

use crossterm::cursor::MoveTo;
use crossterm::cursor::MoveToNextLine;
use crossterm::queue;
use crossterm::style::Color;
use crossterm::style::Print;
#[cfg(not(target_os = "windows"))]
use crossterm::style::ResetColor;
use crossterm::style::SetForegroundColor;
use crossterm::terminal::Clear;
use crossterm::terminal::ClearType;
use std::io::stdout;

/// Text to show as command-line help --help
const COMMANDLINE_HELP: &str = r#"
Hangman is a paper and pencil guessing game for two or more players.  One player
thinks of a word, phrase or sentence and the other tries to guess it by
suggesting letters or numbers, within a certain number of guesses. In this
version for children the computer selects a word, phrase or sentence randomly
out of a word-list defined in a configuration file. In the course of the game
Ascii-Art images - designed for children - are progressively disclosed.

```
-----------------------------------
    ASCII-ART HANGMAN FOR KIDS

             ,.
            (_|,.
           ,' /, )_______
        __j o``-'        `
       (")
        `-j                |
          `-._(           /
             |_\  |--^.  /
            /_]'|_| /_)_/
               /_]'  /_]'

    Lifes:  1       Last guess: 3

     g o o _ _ l _ _ k

    Type a letter then type [Enter]:
-----------------------------------
```

```
    Usage: ascii-hangman
           ascii-hangman [FILE]
           ascii-hangman -h|--help
           ascii-hangman -V|--version
```

`[FILE]` are configuration files containing one word-list hereafter referred to as "secrets" and
optionally one Ascii-Art image.

When no `[FILE]` argument is given, `[FILE]` defaults to `ascii-hangman-words.txt`. In case no
`[FILE]` is found, a template configuration file `ascii-hangman-words.txt` is written into the
current working directory. Multiple `[FILE]`s are concatenated.

`[FILE]` is a UTF-8 YAML formatted file containing 3 different variables:

- `secrets:` is an array of secrets, one per line. A secret is a string, that interprets the `|`
  character as newline and the `_` character as visibility switch. This switch allows to
  disclose a part of the secret when the game starts.
- `traditional:` is an optional boolean variable indicating how the ASCII-art image should be
  disclosed:
  * `true`: the image gets disclosed with every lost life,
  * `false`: the image gets disclosed with every guessed character (default).
- `image: |1` is an optional multiline string providing own ASCII-art. When missing, built-in
  ASCII-art is chosen randomly.
- Lines starting with `#` are ignored.

Example:

```
secrets:
 - guess me
 - "guess me: with colon"
 - line| break
 - _disclose _partly

  # The following is optional.
 traditional: false
  # Optional ASCII-art lines start with 1 space ' '.

 image: |1
    ::
  C|__|
```

A traditional (not built-in) configuration could look like this:

```
secrets:
- guess me
- _good l_uck
- "_der Hund:_| the dog"
- _3*_7_=21_

traditional: true

image: |1
  ______
  |    |
  |    O
  |   /|\
  |    |
  |   / \
__|_____
|      |___
|_________|

```

When the word `secrets` is not found in `[FILE]`, _ASCII-Hangman_ switches into backward
compatibility mode, allowing to use old non-YAML configuration files without modification.
To learn more about the YAML format visit: <https://en.wikipedia.org/wiki/YAML>
"#;

/// Default configuration filename when no filename is given at the command-line.
const PATHSTR: &str = "ascii-hangman-words.txt";
/// Fallback sample configuration when no configuration file can be found.

/// Fallback secret when no configuration file can be found.
const CONF_DEMO: &str = "secrets:\n - \"_Demo: add own words to config file and start a_gain_!\"";

// ------------------ MAIN ---------------------------------------------

/// Reads the configuration file.
pub fn read_config(pathstr: &Path) -> Result<String, io::Error> {
    let mut f = File::open(pathstr)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    Ok(s)
}

/// Writes a sample configuration file on disk. Called when no configuration file can be found.
pub fn write_config_template(pathstr: PathBuf) -> Result<(), io::Error> {
    let mut file = File::create(&pathstr)?;
    file.write_all(CONF_TEMPLATE.as_bytes())?;
    Ok(())
}

/// Starts the game.
/// This is the terminal application
#[allow(unused_labels)]
fn main() {
    // SHOW HELP TEXT
    match env::args().nth(1) {
        Some(ref a) if a == "-h" || a == "--help" => {
            eprintln!("{}", COMMANDLINE_HELP);
            return;
        }
        Some(ref a) if a == "-V" || a == "--version" => {
            eprintln!("{}", VERSION.unwrap());
            return;
        }
        Some(_) | None => {}
    };

    // READ CONFIG

    // Read all config files given on command line
    let mut conf_file_paths = env::args()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<PathBuf>>();

    // if no conf_file_paths are given then use default config path
    if conf_file_paths.is_empty() {
        conf_file_paths.push(PathBuf::from(PATHSTR))
    };

    // read and concatenate all config files given on command line
    let cwd = env::current_dir().unwrap();

    let mut config: String = String::new();
    for conf_file_path in &conf_file_paths {
        let path = conf_file_path;
        let c = match read_config(path) {
            Ok(s) => s,
            Err(_) => {
                match write_config_template(path.to_path_buf()) {
                    Ok(_) => {
                        eprintln!(
                            "As no config-file :\n\
                             \t{:?}\n\
                             was found a template file is written in the \
                             current working directory.\n\
                             \t{:?}\n\n\nPress [Enter] to enter demo mode.",
                            path, cwd
                        );
                        // wait for [Enter] key
                        let s = &mut String::new();
                        io::stdin().read_line(s).unwrap();
                        CONF_DEMO.to_string()
                    }
                    Err(why) => {
                        eprintln!(
                            "Couldn't write ascii-hangman template \
                             config-file:\n\t{:?}\n({})\n\n\
                             Current working directory is:\n\t{:?}\n\n\
                             Press [Enter] to enter demo mode.",
                            path,
                            why.to_string(),
                            cwd
                        );
                        // wait for [Enter] key
                        let s = &mut String::new();
                        io::stdin().read_line(s).unwrap();
                        CONF_DEMO.to_string()
                    }
                }
            }
        };
        config.push_str(&c);
    }

    // INITIALISE THE GAME

    let mut app = match Backend::new(&config) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("ERROR IN CONFIGURATION FILE\n{}", e);

            // wait for [Enter] key
            let s = &mut String::new();
            io::stdin().read_line(s).unwrap();
            process::exit(1);
        }
    };

    app.render();

    // PLAY

    'playing: loop {
        // Read user input
        io::stdout().flush().unwrap();
        // Read next char and send it
        let key = &mut String::new();
        io::stdin().read_line(key).unwrap();

        app.process_user_input(key);
        app.render();
        io::stdout().flush().unwrap();
        let state = app.get_state();
        if state != State::Ongoing {
            if state != State::VictoryGameOver {
                queue!(
                    stdout(),
                    MoveToNextLine(1),
                    Print("Press any key to continue or [Ctrl+C] to quit.")
                )
                .unwrap();
                stdout().flush().unwrap();
            } else {
                break;
            };
        };
    }

    queue!(stdout(), MoveToNextLine(1), Print(AUTHOR)).unwrap();
    stdout().flush().unwrap();
    let key = &mut String::new();
    io::stdin().read_line(key).unwrap();
}

/// Trait to render the text user interface (TUI)
trait Render {
    /// Renders and prints the TUI on the terminal.
    fn render(&self) {}
}

impl Render for Backend {
    fn render(&self) {
        // Disclose parts of the image.

        // Clear all lines in terminal;
        queue!(stdout(), Clear(ClearType::All), MoveTo(0, 0)).unwrap();

        #[cfg(not(windows))]
        queue!(stdout(), ResetColor).unwrap();
        #[cfg(windows)]
        queue!(stdout(), SetForegroundColor(Color::Grey),).unwrap();

        queue!(
            stdout(),
            Print(&TITLE),
            MoveToNextLine(1),
            SetForegroundColor(Color::DarkYellow),
        )
        .unwrap();

        // Print image.
        queue!(stdout(), Print(self.render_image()), MoveToNextLine(1)).unwrap();

        // Print game status.
        #[cfg(not(windows))]
        queue!(stdout(), ResetColor).unwrap();
        #[cfg(windows)]
        queue!(stdout(), SetForegroundColor(Color::Grey),).unwrap();
        queue!(stdout(), Print(self.render_game_lifes()), Print("\t")).unwrap();
        queue!(
            stdout(),
            Print(self.render_game_last_guess()),
            MoveToNextLine(2)
        )
        .unwrap();

        // Print secret.
        #[cfg(not(windows))]
        queue!(stdout(), SetForegroundColor(Color::DarkGreen),).unwrap();
        #[cfg(windows)]
        queue!(stdout(), SetForegroundColor(Color::White),).unwrap();
        queue!(stdout(), Print(self.render_secret()), MoveToNextLine(1)).unwrap();

        // Print instructions.
        #[cfg(not(windows))]
        queue!(stdout(), ResetColor).unwrap();
        #[cfg(windows)]
        queue!(stdout(), SetForegroundColor(Color::Grey),).unwrap();

        queue!(stdout(), Print(self.render_instructions()), Print(" ")).unwrap();
        // Print queued.
        stdout().flush().unwrap();
    }
}
