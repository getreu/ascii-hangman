//! Manages and prints the TUI.
use crate::image::Image;
use crate::Render;
use std::io::{stdout, Write};
extern crate crossterm;
use crossterm::cursor::MoveTo;
use crossterm::cursor::MoveToNextLine;
use crossterm::queue;
use crossterm::style::Color;
use crossterm::style::Print;
use crossterm::style::SetForegroundColor;
use crossterm::terminal::Clear;
use crossterm::terminal::ClearType;

/// Title line.
const TITLE: &str = "ASCII-ART HANGMAN FOR KIDS";

/// Postion of the upper left corner of the image on the screen.
const OFFSET: (usize, usize) = (1, 2);

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

        self.image.render();

        // print message field
        let mut emph = false;
        for line in &mut self.message.lines() {
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
