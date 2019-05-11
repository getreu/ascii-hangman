//! Manages and prints the TUI.
use crate::image::Image;
use crate::Render;

extern crate crossterm;
#[cfg(unix)]
use crossterm::Attribute;
use crossterm::{cursor, terminal, ClearType, Color, Colored};

/// Titleline.
const TITLE: &str = "ASCII-ART HANGMAN FOR KIDS";

/// Postion of the upper left corner of the image on the screen.
const OFFSET: (usize, usize) = (1, 1);

/// State of the TUI.
#[derive(Debug)]
pub struct UserInterface {
    pub image: Image,
    pub message: String,
}

/// Printable representation of the TUI.
impl Render for UserInterface {
    /// Renders and prints the TUI.  It would be more consistent to implement Display for Image,
    /// but crossterm does not support `print!(f, ...)`. Therefor, it is not on option here.
    fn render(&self) {
        let terminal = terminal();
        // Clear all lines in terminal;
        terminal.clear(ClearType::All).unwrap();
        cursor().goto(0, 0).unwrap();

        #[cfg(unix)]
        print!("{}", Attribute::Reset);
        #[cfg(windows)]
        print!("{}", Colored::Fg(Color::Grey));
        println!("{}", &TITLE);

        print!("{}", Colored::Fg(Color::DarkYellow));
        &self.image.render();
        println!("\n");

        terminal.clear(ClearType::FromCursorDown).unwrap();
        // print message field
        let mut emph = false;
        for line in &mut self.message.lines() {
            if line == "" {
                emph = !emph
            };
            if emph {
                #[cfg(unix)]
                print!("{}", Colored::Fg(Color::DarkGreen));
                #[cfg(windows)]
                print!("{}", Colored::Fg(Color::White));
                println!("{}", &line);
            } else {
                #[cfg(unix)]
                print!("{}", Attribute::Reset);
                #[cfg(windows)]
                print!("{}", Colored::Fg(Color::Grey));
                println!("{}", &line);
            }
        }

        #[cfg(Unix)]
        print!("{}", Attribute.Reset);
    }
}

impl UserInterface {
    /// Constructor.
    pub fn new(config: &str) -> Self {
        Self {
            image: Image::new(&config, OFFSET),
            message: String::new(),
        }
    }
}
