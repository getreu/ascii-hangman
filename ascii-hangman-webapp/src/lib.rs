//! This is the wasm web-gui.
//! This file is only compiled for the wasm32 target.

#![cfg(target_arch = "wasm32")]
#![recursion_limit = "512"]

use ascii_hangman_backend::game::State;
use ascii_hangman_backend::Backend;
use ascii_hangman_backend::HangmanBackend;
use ascii_hangman_backend::{AUTHOR, CONF_TEMPLATE, CONF_TEMPLATE_SHORT, TITLE, VERSION};
use wasm_bindgen::prelude::*;
use yew::events::KeyboardEvent;
use yew::prelude::*;
use yew::services::reader::{File, FileData, ReaderService, ReaderTask};
// Disable debugging code.
//use yew::services::ConsoleService;
use yew::services::DialogService;
use yew::{html, Component, ComponentLink, Html, InputData, ShouldRender};

#[derive(Debug)]
pub enum Scene {
    Playground(Backend),
    ConfigureGame,
    GameOver,
}

pub struct GuiState {
    config_text: String,
    guess: String,
}

pub struct Model {
    link: ComponentLink<Self>,
    // Disable debugging code.
    //console: ConsoleService,
    filereader_tasks: Vec<ReaderTask>,
    scene: Scene,
    state: GuiState,
}

#[derive(Debug)]
pub enum Msg {
    SwitchTo(Scene),
    ConfigTextDelete,
    ConfigTextUpdate(String),
    ConfigReady,
    Files(Vec<File>),
    Loaded(FileData),
    UpdateGuess(String),
    Guess,
    Nope,
    NextRound,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let state = GuiState {
            config_text: String::from(CONF_TEMPLATE_SHORT),
            guess: String::new(),
        };

        Model {
            link,
            // Disable debugging code.
            //console: ConsoleService::new(),
            filereader_tasks: vec![],
            scene: Scene::ConfigureGame,
            state,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        let mut new_scene = None;
        match &mut self.scene {
            Scene::Playground(ref mut app) => match msg {
                Msg::SwitchTo(Scene::ConfigureGame) => {
                    new_scene = Some(Scene::ConfigureGame);
                }
                Msg::SwitchTo(Scene::GameOver) => {
                    new_scene = Some(Scene::GameOver);
                }
                Msg::UpdateGuess(val) => {
                    self.state.guess = val.chars().rev().take(1).collect();
                    // Disable debugging code.
                    //self.console.debug(&self.state.guess);
                }
                Msg::Guess => {
                    app.process_user_input(&self.state.guess);
                    self.state.guess = String::new();
                }
                Msg::Nope => {}
                unexpected => {
                    panic!(
                        "Unexpected message when configurations list shown: {:?}",
                        unexpected
                    );
                }
            },
            Scene::ConfigureGame => match msg {
                Msg::ConfigTextUpdate(val) => {
                    self.state.config_text = val;
                }
                Msg::ConfigTextDelete => {
                    self.state.config_text = CONF_TEMPLATE_SHORT.to_string();
                }
                Msg::SwitchTo(Scene::Playground(app)) => {
                    new_scene = Some(Scene::Playground(app));
                }

                Msg::SwitchTo(Scene::GameOver) => {
                    if DialogService::confirm("Do you really want to quit this game?") {
                        new_scene = Some(Scene::GameOver);
                    }
                }

                Msg::Loaded(file) => {
                    if let Ok(s) = std::str::from_utf8(&file.content) {
                        self.state.config_text.push_str(s);
                    } else {
                        DialogService::alert(&format!("Can not read text file: {}", file.name));
                    }
                }
                Msg::Files(files) => {
                    for file in files.into_iter() {
                        let task = {
                            let callback = self.link.callback(Msg::Loaded);
                            ReaderService::read_file(file, callback).unwrap()
                        };
                        self.filereader_tasks.push(task);
                    }
                }
                Msg::ConfigReady => {
                    match Backend::new(self.state.config_text.as_str()) {
                        Ok(app) => {
                            self.link
                                .send_message(Msg::SwitchTo(Scene::Playground(app)));
                        }
                        Err(e) => {
                            DialogService::alert(&format!("Can not parse configuration:\n {}", e))
                        }
                    };
                }
                unexpected => {
                    panic!(
                        "Unexpected message during new config editing: {:?}",
                        unexpected
                    );
                }
            },
            Scene::GameOver => match msg {
                Msg::SwitchTo(Scene::ConfigureGame) => {
                    new_scene = Some(Scene::ConfigureGame);
                }
                unexpected => {
                    panic!("Unexpected message for settings scene: {:?}", unexpected);
                }
            },
        }
        if let Some(new_scene) = new_scene.take() {
            self.scene = new_scene;
        }
        true
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let header = move || -> Html {
            html! { <h1> {TITLE} </h1> }
        };
        let footer = move || -> Html {
            html! { <footer class="footer">
            <a href="../ascii-hangman--manual.html"> {"User Manual"} </a>
            {", "} <a href="../#distribution"> {"Desktop Version"} </a>
            {", "} <a href="../"> {"Documentation"} </a>
            {", "} <a href="https://github.com/getreu/ascii-hangman"> {"Source Code"} </a>
            {", Version "} {VERSION.unwrap()} {", "} {AUTHOR}  </footer> }
        };
        match self.scene {
            Scene::ConfigureGame => html! { <>
                {header()}
                <div class="ascii-hangman-wasm">
                    <div> {"Enter your secrets here:"}</div>
                    <div>
                    <textarea class="conf-text"
                        placeholder=CONF_TEMPLATE
                        cols=80
                        rows=25
                        value=self.state.config_text.clone()
                        oninput=self.link.callback(|e: InputData| Msg::ConfigTextUpdate(e.value)) />
                    </div>
                    <div class="upload-container"> { "or load secrets from files: "}
                        <label class="upload-link" for="upload">{"Upload Files ..."}</label>
                        <input class="custom-file-input" type="file" id="upload" multiple=true onchange=self.link.callback(move |value| {
                                let mut result = Vec::new();
                                if let ChangeData::Files(files) = value {
                                    let files = js_sys::try_iter(&files)
                                        .unwrap()
                                        .unwrap()
                                        .into_iter()
                                        .map(|v| File::from(v.unwrap()));
                                    result.extend(files);
                                }
                                Msg::Files(result)
                            })/>
                    </div>

                    <button disabled=self.state.config_text.is_empty()
                            onclick=self.link.callback(|_| Msg::ConfigTextDelete)>{ "Delete Secrets" }</button>
                    <button disabled=self.state.config_text.is_empty()
                            onclick=self.link.callback(|_| Msg::ConfigReady)>{ "Start Game" }</button>
                </div>
                {footer()}
                </>
            },
            Scene::Playground(ref app) => {
                let secret = app.render_secret();
                let (cols, rows) = dimensions(&secret);
                let secret = secret.trim_end_matches("\n").to_string();
                let image = app.render_image();
                let image = image.trim_end_matches("\n").to_string();
                html! { <>
                    {header()}
                    <div class="ascii-hangman-wasm">
                            <textarea class="image"
                                placeholder="Image"
                                cols=format!("{}", &app.get_image_dimension().0)
                                rows=format!("{}", &app.get_image_dimension().1)
                                value=image
                                readonly=true
                            />
                        <table class="game-status">
                        <tr>
                        <th>
                            { app.render_game_lifes() } { " " }
                        </th>
                        <th>
                            { app.render_game_last_guess() }
                        </th>
                        </tr>
                        </table>
                            <textarea class="secret"
                                cols=format!("{}", cols+1)
                                rows=format!("{}", rows)
                                value=secret
                                readonly=true
                            />
                        <div class="instructions">
                            { app.render_instructions() }
                            <input class="guess"
                                type="text"
                                autofocus=true
                                size=1
                                value=self.state.guess.clone()
                                oninput=self.link.callback(|e: InputData| Msg::UpdateGuess(e.value))
                                onkeypress=self.link.callback(|e: KeyboardEvent| {
                                   if e.key() == "Enter" { Msg::Guess } else { Msg::Nope }
                                }) />

                        </div>
                        <button disabled={app.get_state() == State::Ongoing || app.get_state() == State::VictoryGameOver}
                                onclick=self.link.callback(|_| Msg::Guess)>{ "Continue Game" }</button>
                        <button disabled={app.get_state() != State::VictoryGameOver}
                                onclick=self.link.callback(|_| Msg::SwitchTo(Scene::ConfigureGame))>{ "Reset Game" }</button>
                        <button disabled={app.get_state() != State::VictoryGameOver}
                                onclick=self.link.callback(|_| Msg::SwitchTo(Scene::GameOver))>{ "End Game" }</button>
                    </div>
                    {footer()}
                    </>
                }
            }
            Scene::GameOver => html! { <>
                {header()}
                <div> {"You can now close this window."}</div>
                <div> <label>{ "Bye bye!" } </label>
                <p/>
                    <button onclick=self.link.callback(|_| Msg::SwitchTo(Scene::ConfigureGame))>{ "No, Continue Playing" }</button>
                </div>
                {footer()}
                </>
            },
        }
    }
}

/// Returns the columns and lines of the smallest
/// grid that can display this multi-line string `s`.
pub fn dimensions(s: &str) -> (usize, usize) {
    let mut row = 0;
    let mut col = 0;
    for l in s.lines() {
        let c = l.chars().count();
        if c > col {
            col = c;
        };
        row += 1;
    }
    (col, row)
}

#[wasm_bindgen(start)]
pub fn run_app() {
    App::<Model>::new().mount_to_body();
}

#[cfg(test)]
mod tests {
    use super::*;
    use secret::Secret;
    #[test]
    fn test_dimensions() {
        let secret = Secret::new("Lorem ipsum dolor sit amet, consectetur adipiscing\
         elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam,\
         quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure\
         dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur\
         sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est\
         laborum.");

        //secret.disclose_all();

        assert_eq!(dimensions(format!("{}", secret).as_str()), (68, 22));
        //assert_eq!(format!("{}", secret), String::new());
    }
}
