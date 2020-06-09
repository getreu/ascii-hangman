use crate::dictionary::ConfigParseError;
use crate::dictionary::Dict;
use crate::game::Game;
use crate::game::State;
use crate::image::Image;
use crate::LIVES;

/// State of the application.
#[derive(Debug)]
pub struct Application {
    dict: Dict,
    game: Game,
    image: Image,
}

impl Application {
    /// Initialize the application with config data and start the first game.
    pub fn new(config: &str) -> Result<Self, ConfigParseError> {
        let mut dict = Dict::new(&config)?;
        // A dictionary guaranties to have least one secret.
        let secret = dict.get_random_secret().unwrap();
        let game = Game::new(&secret, LIVES, dict.is_empty());
        let mut image = Image::new(&config)?;
        image.update(&game);
        Ok(Self { dict, game, image })
    }

    /// The user_input is a key stroke. The meaning depends on the game's state:
    /// Either it is a guess or it is the answer to a yes or no question. Returns false if the
    /// game ended and the dictionary is empty or, if the user wants to quit.
    pub fn process_user_input(&mut self, inp: &str) -> bool {
        match self.game.state {
            State::Victory => {
                let a = inp.chars().next().unwrap_or('Y');
                if a == 'N' || a == 'n' {
                    false
                } else {
                    // Start a new game. As we did not get a `State::VictoryGameOver` we know
                    // there is at least one secret left.
                    let secret = self.dict.get_random_secret().unwrap();
                    self.game = Game::new(&secret, LIVES, self.dict.is_empty());
                    self.image.update(&self.game);
                    true
                }
            }

            State::VictoryGameOver => false,

            State::Defeat | State::DefeatGameOver => {
                let a = inp.chars().next().unwrap_or('Y');
                if a == 'N' || a == 'n' {
                    false
                } else {
                    // We will ask this secret again; this way we never end a game with a defeat.
                    self.dict.add((self.game.secret).to_string());
                    // Start a new game. As we just added a secret, we know there is at least one.
                    let secret = self.dict.get_random_secret().unwrap();
                    self.game = Game::new(&secret, LIVES, self.dict.is_empty());
                    self.image.update(&self.game);
                    true
                }
            }
            State::Ongoing => {
                self.game.guess(inp.chars().next().unwrap_or(' '));
                self.image.update(&self.game);
                true
            }
        }
    }

    /// Renders the image. Make sure it is up to date with `self.image.update()`.
    pub fn render_image(&self) -> String {
        format!("{}", self.image)
    }

    /// Renders the partly hidden secret.
    pub fn render_secret(&self) -> String {
        format!("{}", self.game.secret)
    }

    /// Informs about some game statistics.
    pub fn render_game_status(&self) -> String {
        format!("{}", self.game)
    }

    /// Tells the user what to do next.
    pub fn render_instructions(&self) -> String {
        match self.game.state {
            State::Victory => String::from(
                "Congratulations! You won!\n\
                             New game? Type [Y]es or [n]o: ",
            ),
            State::VictoryGameOver => String::from(
                "Congratulations! You won!\n\
                             There are no more secrets to guess. Game over. Press any key.",
            ),
            State::Defeat | State::DefeatGameOver => String::from(
                "You lost.\n\
                             New game? Type [Y]es or [n]o: ",
            ),
            State::Ongoing => String::from("Type a letter, then press [Enter]: "),
        }
    }
}
