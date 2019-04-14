use std::fmt;

use crate::dictionary::CONF_LINE_WORD_MODIFIER__VISIBLE;
const LINE_WIDTH: usize = 20;

#[derive(Debug)]
struct HangmanChar {
    char_: char,
    visible: bool,
}

impl fmt::Display for HangmanChar {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if self.visible {
            write!(f, "{}", self.char_)
        } else {
            write!(f, "_")
        }
    }
}

pub enum State {
    Ongoing,
    Victory,
    Defeat,
}

#[derive(Debug)]
pub struct Game {
    word: Vec<HangmanChar>,
    pub lives: u8,
    pub last_guess: char,
}

impl Game {
    pub fn get_state(&self) -> State {
        if self.lives == 0 {
            State::Defeat
        } else if self.word.iter().all(|c| c.visible) {
            State::Victory
        } else {
            State::Ongoing
        }
    }

    pub fn new(wordstr: &str, l: u8) -> Self {
        // parse wordsstr, filp 'visible' every CONF_LINE_WORD_MODIFIER__VISIBLE
        let w = wordstr
            .chars()
            // for every * found flip v_acc
            .scan(true, |v_acc, c| {
                *v_acc ^= c == CONF_LINE_WORD_MODIFIER__VISIBLE;
                if c == CONF_LINE_WORD_MODIFIER__VISIBLE {
                    Some(None)
                } else {
                    Some(Some(HangmanChar {
                        char_: c,
                        visible: *v_acc,
                    }))
                }
            })
            // ommit None and unwrap
            .filter_map(|s| s)
            //.inspect(|ref x| println!("after scan:\t{:?}", x))
            .collect();

        println!("{:?}", w);

        Self {
            word: w,
            lives: l,
            last_guess: ' ',
        }
    }

    pub fn guess(&mut self, char_: char) {
        if char_ == '\n' {
            return;
        };
        self.last_guess = char_;
        let mut found = false;
        for h_char in &mut self.word {
            if h_char.char_.eq_ignore_ascii_case(&char_) {
                h_char.visible = true;
                found = true;
            }
        }

        if !found {
            self.lives -= 1;
        }

        if self.lives == 0 {
            for hc in &mut self.word {
                hc.visible = true;
            }
        }
    }

    pub fn visible_chars(&self) -> usize {
        self.word.iter().filter(|hc| !hc.visible).count()
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        writeln!(
            f,
            "\x1b[KLives:\t{}\tLast guess: {}\n",
            self.lives, self.last_guess
        )?;

        let mut linebreak = false;
        for (n, c) in self.word.iter().enumerate() {
            if n % LINE_WIDTH == 0 {
                linebreak = true
            };
            if n == 0 {
                linebreak = false
            };
            if linebreak && (c.char_ == ' ') {
                linebreak = false;
                writeln!(f, " {}", c)?
            } else {
                write!(f, " {}", c)?
            }
        }
        writeln!(f)
    }
}
