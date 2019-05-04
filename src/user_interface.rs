use crate::image::Image;
use crate::Render;

extern crate crossterm;

use crossterm::{cursor, terminal, ClearType };

const TITLE: &str = "ASCII-ART HANGMAN FOR KIDS";

#[derive(Debug)]
pub struct UserInterface {
    pub image: Image,
    pub message: String,
}


impl Render for UserInterface {
    fn render(&self) {
        // Clear all lines in terminal;
        let terminal = terminal();
        terminal.clear(ClearType::All).expect("Can not clear terminal.");
        cursor().goto(0, 0).expect("Can not set curson position.");
        terminal.write(&TITLE).expect("Can not write on terminal");
        terminal.write("\n").expect("Can not write on terminal");
        &self.image.render();
        terminal.write("\n\n").expect("Can not write on terminal");
        terminal.clear(ClearType::FromCursorDown).expect("Can not clear current line.");
        terminal.write(&self.message).expect("Can not write on terminal");
     }
}

impl UserInterface {
    pub fn new(config: &str, offset: (usize, usize)) -> Self {
        Self {
            image: Image::new(&config, offset),
            message: String::new(),
        }
    }
}
