//! Manages and prints the TUI.
use crate::game::Game;
use crate::game::State;
use crate::image::Image;
use std::io::{stdout, Write};
extern crate crossterm;
use crate::dictionary::ConfigParseError;
use crossterm::cursor::MoveTo;
use crossterm::cursor::MoveToNextLine;
use crossterm::queue;
use crossterm::style::Color;
use crossterm::style::Print;
use crossterm::style::SetForegroundColor;
use crossterm::terminal::Clear;
use crossterm::terminal::ClearType;
use std::io;

/// Title line.
const TITLE: &str = "ASCII-ART HANGMAN FOR KIDS\n";

/// State of the TUI.
#[derive(Debug)]
pub struct UserInterface {
    pub image: Image,
}

/// Printable representation of the TUI.
impl UserInterface {
    /// Renders and prints the TUI.  It would be more consistent to implement Display for Image,
    /// but crossterm does not support `print!(f, ...)`. Therefor, it is not on option here.
    pub fn render(&mut self, game: &Game) -> String {
        // Disclose parts of the image.
        self.image.update(&game);

        // Clear all lines in terminal;
        queue!(stdout(), Clear(ClearType::All), MoveTo(0, 0)).unwrap();

        #[cfg(not(windows))]
        queue!(stdout(), SetForegroundColor(Color::White),).unwrap();
        #[cfg(windows)]
        queue!(stdout(), SetForegroundColor(Color::Grey),).unwrap();

        queue!(
            stdout(),
            Print(&TITLE),
            MoveToNextLine(1),
            SetForegroundColor(Color::DarkYellow),
        )
        .unwrap();

        println!("{}", self.image);

        // print message field
        let mut emph = false;

        for line in &mut format!("{}\n", &game).lines() {
            if line == "" {
                emph = !emph
            };
            if emph {
                #[cfg(not(windows))]
                queue!(stdout(), SetForegroundColor(Color::DarkGreen),).unwrap();

                #[cfg(windows)]
                queue!(stdout(), SetForegroundColor(Color::White),).unwrap();
            } else {
                #[cfg(not(windows))]
                queue!(stdout(), SetForegroundColor(Color::White),).unwrap();

                #[cfg(windows)]
                queue!(stdout(), SetForegroundColor(Color::Grey),).unwrap();
            }

            // Print message line.
            queue!(stdout(), Print(&line), MoveToNextLine(1)).unwrap();
        }

        // Print queued.
        stdout().flush().unwrap();

        match game.state {
            State::Victory => {
                println!("Congratulations! You won!");
                println!("New game? Type [Y]es or [n]o: ");
            }
            State::VictoryGameOver => {
                println!("Congratulations! You won!");
                println!("There are no more secrets to guess. Game over. Press any key.");
            }
            State::Defeat | State::DefeatGameOver => {
                println!("You lost.");
                println!("New game? Type [Y]es or [n]o: ");
            }
            State::Ongoing => {
                print!("Type a letter, then press [Enter]: ");
            }
        };

        // Read user input
        io::stdout().flush().unwrap();
        // Read next char and send it
        let key = &mut String::new();
        io::stdin().read_line(key).unwrap();
        key.to_string()
    }
}

impl UserInterface {
    /// Constructor.
    pub fn new(config: &str) -> Result<Self, ConfigParseError> {
        Ok(Self {
            image: Image::new(&config)?,
        })
    }
}
