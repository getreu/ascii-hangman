//! This module provides the backend API for the game logic

mod ascii_art;
mod dictionary;
pub mod game;
mod image;
mod secret;
use crate::dictionary::ConfigParseError;
use crate::dictionary::Dict;
use crate::game::Game;
use crate::game::State;
use crate::image::Image;

pub const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
pub const AUTHOR: &str = "(c) Jens Getreu, 2016-2021.";

/// Title line.
pub const TITLE: &str = "ASCII-Hangman for Kids\n";

/// The ASCII-art image should not change too often, to keep the
/// children focused on the words rather then on the image.
/// The disclosing image should be seen as an additional motivation
/// that changes only from time to time. This value determines how many
/// games should start with the same image until it changes.
pub const CHANGE_IMAGE_MAX: usize = 5;

/// Number of wrong guess allowed.
pub const LIVES: u8 = 7;
/// Fallback sample configuration when the desktop application finds no configuration.
/// This is also place holder and default when the web application starts.
pub const CONF_TEMPLATE: &str = "# Add own secrets here; one per line.\r
\r
secrets:\r
- guess me\r
- _good l_uck\r
- \"_der Hund:_| the dog\"\r
- _3*_7_=21_\r
\r
\r
# Uncomment 3 lines to use an optional custom image:\r
\r
#image: |1\r
#   ::\r
# C|__|\r
";

/// Configuration template. This short version is used in the web application.
pub const CONF_TEMPLATE_SHORT: &str = "# Replace the sample secrets with your own; one per line.\r
\r
secrets:\r
- guess me\r
- _good l_uck\r
- \"_der Hund:_| the dog\"\r
- _3*_7_=21_\r
";

/// State of the application.
#[derive(Debug)]
pub struct Backend {
    dict: Dict,
    game: Game,
    image: Image,
    change_image: Option<usize>,
}

/// API to interact with all game logic. This is used by the desktop frontend
/// in `main.rs` or by the web-app frontend in `lib.rs`.
pub trait HangmanBackend {
    /// Initialize the application with config data and start the first game.
    fn new(config: &str) -> Result<Self, ConfigParseError>
    where
        Self: std::marker::Sized;

    /// The user_input is a key stroke. The meaning depends on the game's state:
    fn process_user_input(&mut self, inp: &str);

    /// Renders the image. Make sure it is up to date with `self.image.update()`.
    fn render_image(&self) -> String;

    /// Forward the private image dimension
    fn get_image_dimension(&self) -> (u8, u8);

    /// Renders the partly hidden secret.
    fn render_secret(&self) -> String;

    /// Informs about some game statistics: lifes
    fn render_game_lifes(&self) -> String;

    /// Informs about some game statistics: last guess
    fn render_game_last_guess(&self) -> String;

    /// Tells the user what to do next.
    fn render_instructions(&self) -> String;

    /// Forwards the game's state
    fn get_state(&self) -> State;
}

impl HangmanBackend for Backend {
    fn new(config: &str) -> Result<Self, ConfigParseError> {
        let mut dict = Dict::from(config)?;
        // A dictionary guaranties to have least one secret.
        let secret = dict.get_random_secret().unwrap();
        let game = Game::new(&secret, LIVES, dict.is_empty());
        // We assume, that the configuration file comes with a custom image.
        let mut change_image = None;
        let mut image = Image::from_yaml(config).or_else(|_| {
            // We use our built-in images (first game = 0).
            change_image = Some(0);
            Image::new()
        })?;
        image.update(&game);
        Ok(Self {
            dict,
            game,
            image,
            change_image,
        })
    }

    fn process_user_input(&mut self, inp: &str) {
        match self.game.state {
            State::Victory => {
                // Start a new game. As long as we do not get a `State::VictoryGameOver`, we know
                // that there is at least one secret left.
                let secret = self.dict.get_random_secret().unwrap();
                self.game = Game::new(&secret, LIVES, self.dict.is_empty());
                // We change the image, when we have guessed a certain number of times.
                if let Some(n) = self.change_image {
                    if n == CHANGE_IMAGE_MAX - 1 {
                        // Switch to the next image.
                        if let Ok(new_image) = Image::new() {
                            self.image = new_image;
                        };
                        self.change_image = Some(0);
                    } else {
                        self.change_image = Some(n + 1);
                    };
                };
                self.image.update(&self.game);
            }

            State::VictoryGameOver => {}

            State::Defeat | State::DefeatGameOver => {
                // We will ask this secret again; this way we never end a game with a defeat.
                self.dict.add((self.game.secret).to_raw_string());
                // Start a new game. As we just added a secret, we know there is at least one.
                let secret = self.dict.get_random_secret().unwrap();
                self.game = Game::new(&secret, LIVES, self.dict.is_empty());
                self.image.update(&self.game);
            }
            State::Ongoing => {
                self.game.guess(inp.chars().next().unwrap_or(' '));
                // `guess()` changes the game state:
                self.image.update(&self.game);
            }
        }
    }

    fn render_image(&self) -> String {
        format!("{}", self.image)
    }

    #[allow(dead_code)]
    fn get_image_dimension(&self) -> (u8, u8) {
        self.image.dimension
    }

    fn render_secret(&self) -> String {
        format!("{}", self.game.secret)
    }

    fn render_game_lifes(&self) -> String {
        format!("Lifes: {}", self.game.lifes)
    }

    fn render_game_last_guess(&self) -> String {
        format!("Last guess: {}", self.game.last_guess)
    }

    fn render_instructions(&self) -> String {
        match self.game.state {
            State::Victory => String::from("Congratulations! You won!"),
            State::VictoryGameOver => String::from("Congratulations! You won!"),
            State::Defeat | State::DefeatGameOver => String::from("You lost."),
            State::Ongoing => String::from("Type a letter, then press [Enter]:"),
        }
    }

    fn get_state(&self) -> State {
        self.game.state.clone()
    }
}
