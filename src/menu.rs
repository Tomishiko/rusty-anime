//use console::Key;
//use console::Term;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::queue;
// use crossterm::style::Print;
// use crossterm::style::PrintStyledContent;
// use crossterm::style::SetAttribute;
// use crossterm::style::SetColors;
// use crossterm::style::SetForegroundColor;
use crossterm::event::{poll, read, Event};
use crossterm::style::{Color, Print, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::Clear;
use crossterm::{
    cursor,
    style::{self, Stylize},
    terminal, ExecutableCommand, QueueableCommand,
};
use serde::de::value;
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::fmt::format;
use std::io;
use std::io::stdin;
use std::io::stdout;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::ptr::null;
use std::str::FromStr;

//####################################################### Data types

#[derive(Serialize, Deserialize)]
struct Title {
    names: Name,
    id: i32,
    player: Value,
}
#[derive(Serialize, Deserialize)]
struct Name {
    ru: String,
    en: String,
}
#[derive(Serialize, Deserialize)]
struct Response {
    list: Vec<Title>,
}
pub enum NavType {
    TypeIn,
    Interactive,
}
pub struct MenuNode {
    //pub parent: MenuNode,
    pub show_numbers: bool,
    pub navigation: NavType,
    pub action: fn(&mut App) -> MenuType,
}

pub enum MenuType {
    Main,
    Search,
    List,
    Back,
}
pub struct App {
    pub current_list: Vec<Title>,
    pub menu_stack: Vec<MenuNode>,
}
impl App {
    pub fn new() -> App {
        App {
            current_list: Vec::new(),
            menu_stack: Vec::new(),
        }
    }

    fn fetch_release_list(&mut self) {
        println!("Fetching releases");
        let resp = reqwest::blocking::get(
            "https://api.anilibria.tv/v3/title/updates?filter=names,player,list,id&limit=10",
        )
        .expect("msg")
        .text()
        .expect("msg");

        let mut jsonVal: Value = serde_json::from_str(resp.as_str()).expect("msg");
        self.current_list = serde_json::from_value(jsonVal["list"].take()).expect("parsing error");
        // for i in 0..titles.len(){
        //     println!("{}. {}/{}",i,titles[i].names.ru,titles[i].names.en);
        // }
    }

    pub fn menu_draw_loop(mut selected_option: usize,options: &Vec<String>) -> usize {
        let mut stdout = stdout();
        credentials();
        stdout.queue(Clear(terminal::ClearType::FromCursorDown));

        loop {
            for i in 0..options.len() {
                if i == selected_option {
                    queue!(
                        stdout,
                        SetForegroundColor(Color::Black),
                        SetBackgroundColor(Color::White),
                        Print(format_args!("{}\n", &options[i])),
                        SetForegroundColor(Color::White),
                        SetBackgroundColor(Color::Black),
                    );
                } else {
                    stdout.queue(Print(format_args!("{}\n", &options[i])));
                }
            }
            stdout.flush();

            // match term.read_key() {
            //     Ok(key) => match key {
            //         Key::ArrowDown => {
            //             selected_option += 1;
            //             if selected_option >= options.len() {
            //                 selected_option = 0;
            //             }
            //         }
            //         Key::ArrowUp => {
            //             if selected_option == 0 {
            //                 selected_option = options.len();
            //             }
            //             selected_option -= 1;
            //         }
            //         Key::Escape => {
            //             break;
            //         }
            //         Key::Enter => return selected_option,
            //         _ => selected_option = 0,
            //     },
            //     Err(_) => break,
            // }
            loop {            
                let event = read().unwrap();
                match event {
                    Event::Key(event) if event.kind == KeyEventKind::Press => match event.code {
                        KeyCode::Esc => {
                            return 10;
                        }
                        KeyCode::Down => {
                            selected_option += 1;
                            if selected_option >= options.len() {
                                selected_option = 0;
                            }
                            break;
                        }
                        KeyCode::Up => {
                            if selected_option == 0 {
                                selected_option = options.len();
                            }
                            selected_option -= 1;
                            break;
                        }
                        KeyCode::Enter => { return selected_option}
                        _ => {continue;}
                    },
                    _ => {}
                }
    
            }
            
            stdout.queue(crossterm::cursor::MoveTo(0,5));
        }
    }

    fn chose_releases(&mut self) -> MenuType {
        let mut out_handle = io::stdout().lock();
        for i in 0..self.current_list.len() {
            out_handle
                .write_fmt(format_args!(
                    "{}. {} [{}]\n",
                    i,
                    self.current_list[i].names.ru,
                    self.current_list[i].player["episodes"]["string"]
                ))
                .expect("write error");
        }
        let mut input = String::new();
        io::stdout()
            .write(b"Enter the release number: ")
            .expect("input error");
        out_handle.queue(crossterm::cursor::Show);
        out_handle.flush();
        io::stdin().read_line(&mut input).expect("input error");
        out_handle.queue(crossterm::cursor::Hide);
        let index: usize = input.trim().parse().unwrap();
        out_handle.write_fmt(format_args!(
            "Launching the {}",
            self.current_list[index].names.en
        ));
        out_handle.flush();
        watch_title(&self.current_list[index]);
        //term.read_key();
        read().unwrap();
        return MenuType::Back;
    }
    fn fetch_latest_menu(&mut self) -> MenuType{
        self.fetch_release_list();
        return self.chose_releases();
    }
    fn main_menu(&mut self) -> MenuType {
        let index = App::menu_draw_loop(
            0,
            &vec![
                String::from_str("Fetch todays").unwrap(),
                String::from_str("Search").unwrap(),
            ],
        );
        match index {
            0 => return MenuType::List,
            1 => return MenuType::Search,
            _ => return MenuType::Back,
        }
    }
    fn search_logic(&mut self) -> MenuType {
        let mut stdout = stdout();
        let mut stdin = stdin();
        terminal::Clear(terminal::ClearType::All);
        stdout.write(b"Enter search name: ");
        stdout.queue(cursor::Show);
        stdout.flush();
        let mut search_name = String::new();
        stdin.read_line(&mut search_name);
        let response = search_title(&search_name);
        let mut jsonVal: Value = serde_json::from_str(response.as_str()).expect("Parsing error");
        self.current_list = serde_json::from_value(jsonVal["list"].take()).expect("parsing error");
        self.list_releases();
        stdout.queue(cursor::Hide);
        return MenuType::Back;
    }
    pub fn list_releases(&self) {
        let mut out_handle = io::stdout().lock();
        for i in 0..self.current_list.len() {
            out_handle
                .write_fmt(format_args!(
                    "{}. {} [{}]\n",
                    i,
                    self.current_list[i].names.ru,
                    self.current_list[i].player["episodes"]["string"]
                ))
                .expect("write error");
        }
        let mut input = String::new();
        io::stdout()
            .write(b"Enter the release number: ")
            .expect("input error");
        out_handle.flush();
        io::stdin().read_line(&mut input).expect("input error");
        let index: usize = input.trim().parse().unwrap();
        out_handle.write_fmt(format_args!(
            "Launching the {}",
            self.current_list[index].names.en
        ));
        out_handle.queue(cursor::Hide);
        out_handle.flush();
        watch_title(&self.current_list[index]);

        //return MenuType::Back;
    }
}
pub fn search_title(name: &String) -> String {
    println!("Search interface to be implemented");
    let searchEndpoint = "https://api.anilibria.tv/v3/title/search";
    return reqwest::blocking::get(format!(
        "{searchEndpoint}?limit=-1&order_by=id&search={name}"
    ))
    .expect("msg")
    .text()
    .expect("msg");
}

//##################################################################
fn credentials() {
    // println!("{:^150}","CLI anime episode parser");
    // println!("{:^150}","All voiceover rights reserved by Anilibria team");
    // println!("{:^150}","Check out their website! https://anilibria.top");
    let mut out = io::stdout().lock();
    out.write_fmt(format_args!("{:-<52}\n", ""));
    out.write_fmt(format_args!("{:<120}\n", "CLI anime episode parser"));
    out.write_fmt(format_args!(
        "{:<120}\n",
        "All voiceover rights reserved by Anilibria team"
    ));
    out.write_fmt(format_args!(
        "{:<120}\n",
        "Check out their website! https://anilibria.top"
    ));
    out.write_fmt(format_args!("{:-<52}\n", ""));
    out.flush();
}

pub fn menu_provider(menu_type: MenuType) -> MenuNode {
    match menu_type {
        MenuType::Main => {
            return MenuNode {
                navigation: NavType::Interactive,
                show_numbers: false,
                action: App::main_menu,
            };
        }
        MenuType::List => {
            return MenuNode {
                navigation: NavType::TypeIn,
                show_numbers: true,
                action: App::fetch_latest_menu,
            };
        }
        MenuType::Search => {
            return MenuNode {
                navigation: NavType::TypeIn,
                show_numbers: true,
                action: App::search_logic,
            };
        }
        MenuType::Back => {
            todo!()
        }
    }
}
//TODO:debug interaction logic
fn watch_title(title: &Title) {
    let mut out_handle = io::stdout().lock();
    let mut inputEpisode = String::new();
    loop {
        out_handle.write_fmt(format_args!("{esc}[2J{esc}[1;1H", esc = 27 as char));
        out_handle.flush();
        out_handle.write_fmt(format_args!(
            "Enter episode {}: ",
            title.player["episodes"]["string"]
        ));
        out_handle.queue(cursor::Show);
        out_handle.flush();
        io::stdin().read_line(&mut inputEpisode);
        let mut episode = title.player["list"][inputEpisode.trim()]["hls"]["fhd"]
            .as_str()
            .expect("error parsing json");
        let mut url = format!("https://cache.libria.fun{episode}");
        //let output = Command::new("C:\\Program Files\\KMPlayer 64X\\KMPlayer64.exe")
        //let output = Command::new("C:\\Program Files\\KMPlayer 64X\\KMPlayer64.exe")
        //let mut player = std::env::current_dir().unwrap();
        //player.push("player");
        //player.push("AniPlayer.exe");
        //dbg!(&player);
        out_handle.execute(cursor::Hide);
        let output = Command::new("player/AniPlayer.exe")
            //let output = Command::new("mpv")
            //.arg("--profile=low-latency")
            // --hwdec=auto-safe
            //  .arg("--cache-secs=60")
            // .arg("--hwdec=auto-safe")
            .arg(url)
            .output()
            .expect("player");
        inputEpisode.clear();
    }
}
