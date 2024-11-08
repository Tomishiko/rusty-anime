use crate::api::*;
use crate::menu::*;
use crossterm::QueueableCommand;
use crossterm::{
    cursor,
    event::{read, Event, KeyCode},
    execute, queue,
    style::Print,
    terminal::{self, ClearType},
    ExecutableCommand,
};
use fmt::format;
use io::Read;
use path::PathBuf;
use reqwest::blocking::Client;
use reqwest::header;
use reqwest::header::HeaderValue;
use reqwest::header::ACCEPT;
use reqwest::header::USER_AGENT;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::value::{to_raw_value, RawValue};
use std::{
    io::{stdin, Stdout, Write},
    process::Command,
    str::FromStr,
    *,
};
pub enum NavType {
    TypeIn,
    Interactive,
}
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
    config: Config,
    client: Client,
}

impl App {
    pub fn new(stdout: Stdout, config: Config, dirPath: PathBuf) -> App {
        let mut headers = header::HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(USER_AGENT, HeaderValue::from_static("AnimeR/1.0"));
        let mut app = App {
            current_list: Vec::new(),
            menu_stack: Vec::new(),
            out_handle: stdout,
            client: Client::builder().default_headers(headers).build().unwrap(),
            config: config,
            dirPath: dirPath,
        };
        app.out_handle.execute(terminal::EnterAlternateScreen);

        return app;
    }

    pub fn credentials(&mut self) {
        //let mut out = io::stdout().lock();

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
        );
        self.out_handle.flush();
    }
    pub fn fetch_latest_menu(&mut self) -> MenuType {
        queue!(
            self.out_handle,
            cursor::MoveTo(0, 5),
            cursor::Hide,
            terminal::Clear(ClearType::FromCursorDown),
            Print("Fetching recent releases...\n")
        );
        self.out_handle.flush();

        let mut page: u32 = 1;
        let mut max_page: u32 = 1;
        'paging: loop {
            match fetch_updates_list(page, &self.config.api.url, &self.client) {
                Err(err) => {
                    self.out_handle
                        .execute(Print(format!("Failed to fetch latest: {}\n", err)));
                    //wait for user input
                    loop {
                        self.out_handle.execute(cursor::Hide);
                        if read().unwrap() == Event::Key(KeyCode::Enter.into()) {
                            break;
                        }
                    }
                    return MenuType::Back;
                }
                Ok(val) => {
                    self.current_list = val.1;
                    max_page = val.0.allPages;
                    let options = self.build_release_list();
                    'redraw: loop {
                        let (action, selection) = interactive_menu(
                            &options,
                            self,
                            MenuType::List,
                            Some((page, max_page)),
                        );

                        match action {
                            UserAction::Select => self.watch_title(selection.unwrap()),
                            UserAction::Back => {
                                return MenuType::Back;
                            }
                            UserAction::PageForward => {
                                page += 1;
                                queue!(
                                    self.out_handle,
                                    terminal::Clear(ClearType::FromCursorDown),
                                    Print("Loading...")
                                );
                                self.out_handle.flush();
                                continue 'paging;
                            }
                            UserAction::PageBackward => {
                                page -= 1;
                                queue!(
                                    self.out_handle,
                                    terminal::Clear(ClearType::FromCursorDown),
                                    Print("Loading...")
                                );
                                self.out_handle.flush();
                                continue 'paging;
                            }
                        }
                    }
                }
            }
        }
    }
    pub fn main_menu(&mut self) -> MenuType {
        let (action, index) = interactive_menu(
            &vec![
                String::from_str("Fetch todays\n").unwrap(),
                String::from_str("Search\n").unwrap(),
            ],
            self,
            MenuType::Main,
            None,
        );
        if index.is_none() {
            return MenuType::Back;
        }
        match index.unwrap() {
            0 => return MenuType::List,
            1 => return MenuType::Search,
            _ => return MenuType::Back,
        }
    }
    pub fn search_prompt(&mut self) -> MenuType {
        let stdin = stdin();
        queue!(
            self.out_handle,
            cursor::MoveTo(0, 5),
            terminal::Clear(terminal::ClearType::FromCursorDown),
            Print("Enter search name: "),
            cursor::Show
        );
        self.out_handle.flush();
        match read_line_inter(&mut self.out_handle) {
            Some(value) => self.search_logic(&value),
            None => return MenuType::Back,
        }
    }
    pub fn search_logic(&mut self, search_name: &String) -> MenuType {
        execute!(self.out_handle, Print("\nLoading..."), cursor::Hide);

        let mut page = 1;
        let mut max_page = 1;
        loop {
            match search_title(&search_name, page, &self.config.api.url, &self.client) {
                Ok(val) => {
                    self.current_list = val;
                    max_page = 1;
                }
                Err(err) => {
                    self.out_handle.execute(Print(format_args!(
                        "Failed to fetch, status code {}\n",
                        err.status().unwrap()
                    )));
                    return MenuType::Back;
                }
            };

            self.out_handle.execute(cursor::Hide);
            let menu_options = self.build_release_list();
            let (action, selection) =
                interactive_menu(&menu_options, self, MenuType::List, Some((page, max_page)));
            match action {
                UserAction::Select => self.watch_title(selection.unwrap()),
                UserAction::Back => {}
                UserAction::PageForward => {
                    page += 1;
                    queue!(
                        self.out_handle,
                        terminal::Clear(ClearType::FromCursorDown),
                        Print("Loading...")
                    );
                    self.out_handle.flush();
                    continue;
                }
                UserAction::PageBackward => {
                    page -= 1;
                    queue!(
                        self.out_handle,
                        terminal::Clear(ClearType::FromCursorDown),
                        Print("Loading...")
                    );
                    self.out_handle.flush();
                    continue;
                }
            }
            return MenuType::Back;
        }
    }
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
                "{}. \"{}\" [{}]\n",
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

    fn watch_title(&mut self, selected_option: usize) {
        let title = &self.current_list[selected_option];
        let mut inputEpisode = String::with_capacity(10);
        let mut playlist = get_title_playlist(title.id, &self.client, &self.config.api.url)
            .expect("failed to get titles playlist");
        queue!(
            self.out_handle,
            cursor::MoveTo(0, 5),
            terminal::Clear(ClearType::FromCursorDown)
        );
        loop {
            //self.out_handle.write_fmt(format_args!("{esc}[2J{esc}[1;1H", esc = 27 as char));   // what the fuck?
            queue!(
                self.out_handle,
                cursor::MoveTo(0, 5),
                Print(format!(
                    "Watching now: \"{}\"  [{}]\nEnter episode to watch: ",
                    title.names[0],
                    title.series.as_ref().unwrap_or(&String::new()),
                )),
                cursor::Show
            )
            .unwrap();
            self.out_handle.flush();
            //io::stdin().read_line(&mut inputEpisode);

            //TODO this is dogshit, refactor for your life!
            //#########################################################################
            let input = read_line_inter(&mut self.out_handle);
            if input.is_none() {
                break;
            }
            let temp = input.unwrap();
            let args: Vec<&str> = temp.split_whitespace().collect();
            let mut episode: &String;
            let mut index: usize = 0;
            // Episode number
            match args[0].parse::<usize>() {
                Ok(value) => {
                    // Episodes placed from new to old in api response, so we have to  invert index
                    index = playlist.len() - value;
                }
                Err(_) => {
                    self.out_handle.execute(Print("Wrong episode number\n"));
                    _ = io::stdin().read(&mut [0u8]);
                    continue;
                }
            }
            // Episode string source
            if args.len() == 1 {
                if playlist[index].fullhd.is_some() {
                    episode = &playlist[index].fullhd.as_ref().unwrap();
                } else if playlist[index].hd.is_some() {
                    episode = &playlist[index].hd.as_ref().unwrap();
                } else {
                    episode = &playlist[index].hd.as_ref().unwrap();
                }

                //episode = title.playlist[0].fullhd.ex;
            } else {
                match args[1].trim() {
                    "fullhd" => {
                        if !playlist[index].fullhd.is_some() {
                            self.out_handle
                                .execute(Print("\nNo source available of this quality"));
                            continue;
                        }
                        episode = &playlist[index].fullhd.as_ref().unwrap();
                    }
                    "hd" => {
                        if !playlist[index].hd.is_some() {
                            self.out_handle
                                .execute(Print("\nNo source available of this quality"));
                            continue;
                        }
                        episode = &playlist[index].hd.as_ref().unwrap();
                    }
                    "sd" => {
                        if !playlist[index].sd.is_some() {
                            self.out_handle
                                .execute(Print("\nNo source available of this quality"));
                            continue;
                        }
                        episode = &playlist[index].sd.as_ref().unwrap();
                    }
                    _ => {
                        self.out_handle
                            .execute(Print("\nWrong quality annotation, try one of fullhd/hd/sd"));
                        continue;
                    }
                }
            }
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

            // dbg!(&skips);
            // dbg!(&episode);
            // dbg!(&name);

            let output = Command::new(&self.config.player)
                .arg(skips)
                .arg(name)
                .arg(episode)
                //.arg(format!("{}-episode_name", &title.names[0]))
                //.arg("demuxer-lavf-o=seg_max_retry=10")
                .output()
                .expect("player");
            inputEpisode.clear();
            //dbg!(output);
        }
        execute!(self.out_handle, cursor::MoveTo(0, 5), cursor::Hide);
    }
}
