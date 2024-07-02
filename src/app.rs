use crate::api::*;
use crate::menu::*;
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::style::{Color, Print, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{Clear, ClearType};
use crossterm::{cursor, execute, queue, terminal, ExecutableCommand, QueueableCommand};
use reqwest::blocking::Client;
use std::io;
use std::io::{stdin, stdout, Write};
use std::process::Command;
use std::str::FromStr;
use std::*;
pub enum NavType {
    TypeIn,
    Interactive,
}
pub struct App {
    pub current_list: Vec<Title>,
    pub menu_stack: Vec<MenuNode>,
    pub out_handle: io::Stdout,
    client: Client,
}
impl Drop for App {
    fn drop(&mut self) {
        self.out_handle.execute(terminal::LeaveAlternateScreen);
    }
}
impl App {
    pub fn new() -> App {
        let mut app = App {
            current_list: Vec::new(),
            menu_stack: Vec::new(),
            out_handle: io::stdout(),
            client: Client::builder().build().unwrap(),
        };
        app.out_handle.execute(terminal::EnterAlternateScreen);
        credentials(&mut app.out_handle);

        return app;
    }

    // pub fn menu_draw_loop(mut selected_option: usize, options: &Vec<String>) -> usize {
    //     let mut stdout = stdout().lock();
    //     stdout.queue(cursor::MoveTo(0, 5));
    //     queue!(
    //         stdout,
    //         cursor::Hide,
    //         terminal::Clear(ClearType::FromCursorDown)
    //     );
    //     loop {
    //         for i in 0..options.len() {
    //             if i == selected_option {
    //                 queue!(
    //                     stdout,
    //                     SetForegroundColor(Color::Black),
    //                     SetBackgroundColor(Color::White),
    //                     Print(format_args!("{}\n", &options[i])),
    //                     SetForegroundColor(Color::White),
    //                     SetBackgroundColor(Color::Black),
    //                 );
    //             } else {
    //                 stdout.queue(Print(format_args!("{}\n", &options[i])));
    //             }
    //         }
    //         stdout.queue(Print("\n\nEnter: select\tEsc: back\n"));
    //         stdout.flush();

    //         loop {
    //             let event = read().unwrap();
    //             match event {
    //                 Event::Key(event) if event.kind == KeyEventKind::Press => match event.code {
    //                     KeyCode::Esc => {
    //                         return 10;
    //                     }
    //                     KeyCode::Down => {
    //                         selected_option += 1;
    //                         if selected_option >= options.len() {
    //                             selected_option = 0;
    //                         }
    //                         break;
    //                     }
    //                     KeyCode::Up => {
    //                         if selected_option == 0 {
    //                             selected_option = options.len();
    //                         }
    //                         selected_option -= 1;
    //                         break;
    //                     }
    //                     KeyCode::Enter => return selected_option,
    //                     _ => {
    //                         continue;
    //                     }
    //                 },
    //                 _ => {}
    //             }
    //         }

    //         stdout.queue(crossterm::cursor::MoveTo(0, 5));
    //     }
    // }
    pub fn fetch_latest_menu(&mut self) -> MenuType {
        queue!(
            self.out_handle,
            cursor::MoveTo(0, 5),
            terminal::Clear(ClearType::FromCursorDown),
            Print("Fetching recent releases...\n")
        );

        let mut page: u32 = 1;
        let mut max_page: u32 = 1;
        loop {
            match fetch_updates_list(page) {
                Err(err) => {
                    self.out_handle.execute(Print(format!(
                        "Failed to fetch, status code {}\n",
                        err.status().unwrap()
                    )));
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
                    max_page = val.0.pages;
                    let options = self.build_release_list();
                    let (action, selection) =
                        interactive_menu(&options, self, MenuType::List, Some((page, max_page)));

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
        match index.unwrap() {
            0 => return MenuType::List,
            1 => return MenuType::Search,
            _ => return MenuType::Back,
        }
    }
    pub fn search_logic(&mut self) -> MenuType {
        let stdin = stdin();
        queue!(
            self.out_handle,
            cursor::MoveTo(0, 5),
            terminal::Clear(terminal::ClearType::FromCursorDown),
        );

        self.out_handle.write(b"Enter search name: ");
        self.out_handle.queue(cursor::Show);
        self.out_handle.flush();
        let mut search_name = String::new();
        stdin.read_line(&mut search_name);
        let mut page = 1;
        let mut max_page = 1;
        loop {
            match search_title(&search_name) {
                Ok(val) => {
                    self.current_list = val.1;
                    max_page = val.0.pages;
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
        for i in 0..self.current_list.len() {
            list.push(format!(
                "{}. \"{}\" [{}-{}]\n",
                i + 1,
                self.current_list[i].names.ru,
                self.current_list[i].player["episodes"]["first"],
                self.current_list[i].player["episodes"]["last"]
            ));
        }
        return list;
    }

    fn watch_title(&mut self, selected_option: usize) {
        let title = &self.current_list[selected_option];
        let mut inputEpisode = String::new();
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
                    "Watching now: \"{}\"  [{}-{}]\nEnter episode to watch: ",
                    title.names.ru,
                    title.player["episodes"]["first"],
                    title.player["episodes"]["last"],
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
            //TODO: maybe add fhd/hd parameters parsing
            let temp = input.unwrap();
            let args:Vec<&str> = temp.split_whitespace().collect();
            let mut episode = "";
            if args.len() == 1{
                episode = title.player["list"][args[0].trim()]["hls"]["fhd"]
                .as_str()
                .expect("error parsing json");

            }
            else {
                episode = title.player["list"][args[0].trim()]["hls"][args[1].trim()]
                .as_str().unwrap();
                
            }
            //###################################################################
            let mut url = format!("{API_PLAYER_CACHE}{episode}");
            self.out_handle.execute(cursor::Hide);
            //let output = Command::new("player/AniPlayer.exe")
            let output = Command::new("mpv").arg(url).output().expect("player");
            inputEpisode.clear();
            self.out_handle.execute(cursor::MoveTo(0, 5));
        }
    }
}
