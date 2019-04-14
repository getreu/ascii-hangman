use crate::image::Image;
use std::fmt;

const TITLE: &str = "ASCII-ART HANGMAN FOR KIDS";

#[derive(Debug)]
pub struct UserInterface {
    pub image: Image,
    pub message: String,
}

impl fmt::Display for UserInterface {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "\x1b[2J\x1b[0;0f{}\n{}\n\n{}",
            TITLE, self.image, self.message
        )
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
