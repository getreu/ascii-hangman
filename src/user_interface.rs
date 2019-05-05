use crate::image::Image;
use crate::Render;

extern crate crossterm;
#[cfg(unix)]
use crossterm::Attribute;
use crossterm::{cursor, terminal, ClearType, Color, Colored};

const TITLE: &str = "ASCII-ART HANGMAN FOR KIDS";

#[derive(Debug)]
pub struct UserInterface {
    pub image: Image,
    pub message: String,
}

impl Render for UserInterface {
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
    pub fn new(config: &str, offset: (usize, usize)) -> Self {
        Self {
            image: Image::new(&config, offset),
            message: String::new(),
        }
    }
}
