use crate::api::*;
use crate::menu::*;
use crossterm::{
    cursor,
    event::{read, Event, KeyCode},
    execute, queue,
    style::Print,
    terminal, ExecutableCommand,
};
use io::Read;
use path::PathBuf;
use process::{Child, ChildStdin, ChildStdout, Stdio};
use reqwest::{
    blocking::Client,
    header::{self, HeaderValue, ACCEPT, USER_AGENT},
};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::Write;
use std::{io::Stdout, process::Command, str::FromStr, *};
use tui::backend::CrosstermBackend;

#[derive(Serialize, Deserialize)]
pub struct Config {
    api: Api,
    session_id: String,
    player: String,
    original_names: bool,
}
#[derive(Serialize, Deserialize)]
pub struct Api {
    url: String,
    player: String,
}
pub struct App {
    pub current_list: Vec<Title>,
    pub menu_stack: Vec<MenuNode>,
    pub out_handle: Stdout,
    pub dirPath: PathBuf,
    pub proc: Option<Child>,
    pub terminal: tui::Terminal<CrosstermBackend<Stdout>>,
    childStdin: Option<ChildStdin>,
    childStdout: Option<ChildStdout>,
    config: Config,
    client: Client,
}

impl App {
    pub fn new(
        stdout: Stdout,
        config: Config,
        dirPath: PathBuf,
        terminal: tui::Terminal<CrosstermBackend<Stdout>>,
    ) -> App {
        let mut headers = header::HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(USER_AGENT, HeaderValue::from_static("AnimeR/1.0"));

        let mut app: App;
        // If user is not using default player, we don't need to manage subprocess
        if matches!(config.player.as_str(), "rustyplayer.exe") {
            let mut proc = Command::new(&config.player)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .expect("error running subprocess");
            app = App {
                current_list: Vec::new(),
                menu_stack: Vec::new(),
                out_handle: stdout,
                childStdin: Some(proc.stdin.take().unwrap()),
                childStdout: Some(proc.stdout.take().unwrap()),
                client: Client::builder().default_headers(headers).build().unwrap(),
                config: config,
                dirPath: dirPath,
                proc: Some(proc),
                terminal: terminal,
            };
        } else {
            app = App {
                current_list: Vec::new(),
                menu_stack: Vec::new(),
                out_handle: stdout,
                childStdin: None,
                childStdout: None,
                client: Client::builder().default_headers(headers).build().unwrap(),
                config: config,
                dirPath: dirPath,
                proc: None,
                terminal: terminal,
            };
        }
        app.out_handle
            .execute(terminal::EnterAlternateScreen)
            .expect("Unexpected issue when leaving alternate screen");

        return app;
    }

    pub fn credentials(&mut self) {
        queue!(
            self.out_handle,
            Print(format_args!("{:-<52}\n", "")),
            Print(format_args!("{:<120}\n", "CLI anime episode parser")),
            Print(format_args!(
                "{:<120}\n",
                "All voiceover rights reserved by Anilibria team"
            )),
            Print(format_args!(
                "{:<120}\n",
                "Check out their website! https://anilibria.top"
            )),
            Print(format_args!("{:-<52}\n", ""))
        )
        .expect("Could not write to stdout");
        self.out_handle.flush().expect("Could not write to stdout");
    }
    fn error_msg(&mut self, err: String) {
        self.terminal.clear().expect("Could not clear terminal");
        execute!(
            self.out_handle,
            cursor::MoveTo(0, 0),
            Print(format!(
                "Failed to fetch latest: {err}\nPress enter to continue"
            ))
        )
        .expect("Could not write to stdout");
        //wait for user input
        loop {
            if read().unwrap() == Event::Key(KeyCode::Enter.into()) {
                break;
            }
        }
        self.out_handle
            .execute(terminal::Clear(terminal::ClearType::All))
            .expect("Unable to clear stdout");
    }
    pub fn fetch_latest_menu(&mut self) -> MenuType {
        //self.out_handle.flush();

        let mut page: u32 = 1;
        let mut max_page: u32 = 1;
        loop {
            match fetch_updates_list(page, &self.config.api.url, &self.client) {
                Err(err) => {
                    self.error_msg(err.to_string());
                    return MenuType::Back;
                }
                Ok(val) => {
                    self.current_list = val.1;
                    max_page = val.0.allPages;
                    let options = self.build_release_list();
                    let (action, selection) =
                        interactive_menu(&options, self, "Latest releases", Some((page, max_page)));

                    match action {
                        UserAction::Select => self.watch_title(selection.unwrap()),
                        UserAction::Back => {
                            return MenuType::Back;
                        }
                        UserAction::PageForward => {
                            page += 1;
                            continue;
                        }
                        UserAction::PageBackward => {
                            page -= 1;
                            continue;
                        }
                    }
                }
            }
        }
    }
    // Main menu logic
    pub fn main_menu(&mut self) -> MenuType {
        let (action, index) = interactive_menu(
            &vec![
                String::from_str("1. Fetch todays").unwrap(),
                String::from_str("2. Search").unwrap(),
            ],
            self,
            " Main menu ",
            None,
        );
        if matches!(action, UserAction::Back) {
            return MenuType::Back;
        } else {
            match index.unwrap() {
                0 => return MenuType::List,
                1 => return MenuType::Search,
                _ => return MenuType::Back,
            }
        }
    }
    // Provides search prompt
    pub fn search_prompt(&mut self) -> MenuType {
        match input_menu(&mut self.terminal, "Enter search name: ", " Search ") {
            Some(value) => self.search_logic(&value),
            None => return MenuType::Back,
        }
    }
    // Search logic
    pub fn search_logic(&mut self, search_name: &String) -> MenuType {
        let mut page = 1;
        let mut max_page;

        // Paging loop
        loop {
            match search_title(&search_name, page, &self.config.api.url, &self.client) {
                Ok(val) => {
                    self.current_list = val;
                    max_page = 1;
                }
                Err(err) => {
                    self.error_msg(err.to_string());
                    return MenuType::Back;
                }
            };

            let menu_options = self.build_release_list();
            // we dont want to fetch same result again, so return here
            loop {
                let (action, selection) = interactive_menu(
                    &menu_options,
                    self,
                    " Search result ",
                    Some((page, max_page)),
                );
                match action {
                    UserAction::Select => self.watch_title(selection.unwrap()),
                    UserAction::Back => return MenuType::Back,
                    UserAction::PageForward => {
                        page += 1;
                        break; // To paging loop
                    }
                    UserAction::PageBackward => {
                        page -= 1;
                        break; // To paging loop
                    }
                }
            }
        }
    }
    // Builds a vector of formatted, user-friendly options from received server response
    fn build_release_list(&self) -> Vec<String> {
        let mut list: Vec<String> = Vec::with_capacity(self.current_list.len());
        let localization: usize;

        if self.config.original_names {
            localization = 1;
        } else {
            localization = 0;
        }
        for i in 0..self.current_list.len() {
            list.push(format!(
                "{}. \"{}\" [{}]",
                i + 1,
                self.current_list[i].names[localization],
                self.current_list[i]
                    .series
                    .as_ref()
                    .unwrap_or(&String::new())
            ));
        }
        return list;
    }

    // External player interaction logic
    fn watch_title(&mut self, selected_option: usize) {
        let title = &self.current_list[selected_option];
        let playlist = get_title_playlist(title.id, &self.client, &self.config.api.url)
            .expect("failed to get titles playlist");
        // Chose episode number
        loop {
            let input = input_menu(
                &mut self.terminal,
                format!(
                    "Enter episoed number [{}]: ",
                    title.series.as_ref().unwrap_or(&String::new()),
                )
                .as_str(),
                title.names[0].as_str(),
            );
            if input.is_none() {
                break;
            }
            let temp = input.unwrap();
            let episode_input: Vec<&str> = temp.split_whitespace().collect();
            let episode: &String;
            let index: usize;
            // Parse episode number
            match episode_input[0].parse::<usize>() {
                Ok(value) => {
                    // Episodes placed from new to old in api response, so we have to  invert index
                    index = playlist.len() - value;
                }
                Err(_) => {
                    self.out_handle
                        .execute(Print("Wrong episode number\n"))
                        .expect("Unexpected error happened");
                    _ = io::stdin().read(&mut [0u8]);
                    continue;
                }
            }
            // Episode source string
            // If did not get quality input, choose best possible
            if episode_input.len() == 1 {
                if playlist[index].fullhd.is_some() {
                    episode = &playlist[index].fullhd.as_ref().unwrap();
                } else if playlist[index].hd.is_some() {
                    episode = &playlist[index].hd.as_ref().unwrap();
                } else {
                    episode = &playlist[index].sd.as_ref().unwrap();
                }
            } else {
                // if we've got quality input, try to match it
                match episode_input[1].trim() {
                    "fullhd" => {
                        if !playlist[index].fullhd.is_some() {
                            self.out_handle
                                .execute(Print("\nNo source available of this quality"))
                                .expect("Unexpected error happened");
                            continue;
                        }
                        episode = &playlist[index].fullhd.as_ref().unwrap();
                    }
                    "hd" => {
                        if !playlist[index].hd.is_some() {
                            self.out_handle
                                .execute(Print("\nNo source available of this quality"))
                                .expect("Unexpected error happened");
                            continue;
                        }
                        episode = &playlist[index].hd.as_ref().unwrap();
                    }
                    "sd" => {
                        if !playlist[index].sd.is_some() {
                            self.out_handle
                                .execute(Print("\nNo source available of this quality"))
                                .expect("Unexpected error happened");
                            continue;
                        }
                        episode = &playlist[index].sd.as_ref().unwrap();
                    }
                    _ => {
                        self.out_handle
                            .execute(Print("\nWrong quality annotation, try one of fullhd/hd/sd"))
                            .expect("Unexpected error happened");
                        continue;
                    }
                }
            }
            // If using default player and its instantiated
            if self.proc.is_some() {
                let mut child_stdin = self.childStdin.as_ref().unwrap();
                let child_stdout = self.childStdout.as_mut().unwrap();
                // Skips input building
                let mut skips = String::with_capacity(30);
                skips.push('[');
                if playlist[index].skips.opening.len() == 2 {
                    skips.push_str(&format!(
                        "{},{}",
                        playlist[index].skips.opening[0], playlist[index].skips.opening[1]
                    ));
                }
                if playlist[index].skips.ending.len() == 2 {
                    skips.push_str(&format!(
                        ",{},{}",
                        playlist[index].skips.ending[0], playlist[index].skips.ending[1]
                    ));
                }
                skips.push(']');
                let name = format!(
                    "{} - {}",
                    playlist[index].title,
                    playlist[index].name.as_ref().unwrap_or(&String::new())
                );
                let title_name = title.names[0].as_str();
                let data = format!(
                    r##"{{"source":"{episode}","skips":{skips},"episode_title":"{name}","title":"{title_name}"}}"##
                );
                let length = data.len() as u32;
                child_stdin
                    .write_all(&length.to_le_bytes())
                    .expect("Unable to write to child's stdin");
                child_stdin
                    .write_all(data.as_bytes())
                    .expect("Unable to write to child's stdin");
                child_stdout
                    .read(&mut [0u8])
                    .expect("Unable to read child's stdout");
            } else {
                let player_output = Command::new(self.config.player.as_str())
                    .arg(episode)
                    .output();
            }
        }
    }
}
