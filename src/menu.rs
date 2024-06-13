use console::colors_enabled_stderr;
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
use crossterm::terminal::{Clear,ClearType};
use crossterm::{
    cursor,
    style::{self, Stylize},
    terminal, ExecutableCommand, QueueableCommand,
};
use reqwest::Client;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::format;
use std::io;
use std::io::stdin;
use std::io::stdout;
use std::io::Cursor;
use std::io::Stdout;
use std::io::StdoutLock;
use std::io::Write;
use std::option;
use std::process::Command;
use std::str::FromStr;

//####################################################### Data types
const API_HOST:&str = "https://api.anilibria.tv/v3";
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
    out_handle:io::Stdout,
    client:Client,
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
            out_handle:io::stdout(),
            client:Client::builder().build().unwrap(),
        };
        app.out_handle.execute(terminal::EnterAlternateScreen);
        credentials(&mut app.out_handle);

        return app;
    }

    fn fetch_updates_list(&mut self,page:u8) -> Result<(),reqwest::Error>{
        //println!("Fetching releases");
        let resp = reqwest::blocking::get(
            format!("{API_HOST}/v3/title/updates?filter=names,player,list,id&limit=9&page={page}"),
        )?;
        if resp.status()!= StatusCode::OK{
            self.out_handle.write_fmt(format_args!("Failed to fetch, status code {}\n",resp.status()));
            return  Err(resp.error_for_status().err().unwrap());
        }
        //dbg!(&resp);
        let resp = resp
        .text()?;

        let mut jsonVal: Value = serde_json::from_str(resp.as_str()).expect("error when parsing response");
        self.current_list = serde_json::from_value(jsonVal["list"].take()).expect("error when parsing response");
        return Ok(());
    }

    pub fn menu_draw_loop(mut selected_option: usize,options: &Vec<String>) -> usize {

        let mut stdout = stdout().lock();
        stdout.queue(cursor::MoveTo(0,5));
        queue!(stdout,cursor::Hide,terminal::Clear(ClearType::FromCursorDown));
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
            stdout.queue(Print("\n\nEnter: select\tEsc: back\n"));
            stdout.flush();

            
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
    fn choose_releases(&mut self) -> MenuType {
        //let mut out_handle = io::stdout().lock();
        // Old input******************************************
        // for i in 0..self.current_list.len() {
        //     self.out_handle
        //         .write_fmt(format_args!(
        //             "{}. {} [{}]\n",
        //             i,
        //             self.current_list[i].names.ru,
        //             self.current_list[i].player["episodes"]["string"]
        //         ))
        //         .expect("write error");
        // }
        // let mut input = String::new();
        // io::stdout()
        //     .write(b"Enter the release number: ")
        //     .expect("input error");
        // let start_position = cursor::position().unwrap();
        // self.out_handle.queue(crossterm::cursor::Show);
        // self.out_handle.flush();
        // let mut esc_flag = false;
        // io::stdout().flush();
        // self.out_handle.execute(cursor::EnableBlinking);
        // io::stdin().read_line(&mut input);

        // Custom keyboard reading*************************************************
        // loop {
        //     let event=  read().unwrap();
        //     if event == Event::Key(KeyCode::Esc.into()){
        //         return MenuType::Back;
        //     }
        //     match event {
        //         Event::Key(KeyEvent { 
        //             code:KeyCode::Enter,
        //             kind:KeyEventKind::Press,
        //             ..
        //         })=>{
        //             if input.len()!=0 { break; }
        //         }
        //         Event::Key(KeyEvent { 
        //             code:KeyCode::Char(c),
        //             kind:KeyEventKind::Press,
        //             ..
        //         })=>{
        //             input.push(c);
        //             self.out_handle.execute(Print(c));
        //         }
        //         Event::Key(KeyEvent { 
        //             code:KeyCode::Backspace,
        //             kind:KeyEventKind::Press,
        //             ..
        //         })=>{
        //             if cursor::position().unwrap() != start_position{
        //                 input.pop();
        //                 queue!(self.out_handle,
        //                     Print("\u{8} \u{8}"),);
        //                 self.out_handle.flush();
        //             }
                    
        //         }
        //         _ => {}
        //     }
        // }
    


        

        //io::stdin().read_line(&mut input).expect("input error");
        //self.out_handle.queue(crossterm::cursor::Hide);
        // if(!esc_flag){
        //     let index: usize = input.trim().parse().unwrap();
        //     self.out_handle.write_fmt(format_args!(
        //         "Launching the {}",
        //         self.current_list[index].names.en
        //     ));
        //     self.out_handle.flush();
        //     watch_title(&self.current_list[index]);
        //     //term.read_key();
        //     read().unwrap();
        // }
        let options:Vec<String> = self.current_list
                        .iter()
                        .map(|x| x.names.ru.clone())
                        .collect();
        let index = App::menu_draw_loop(0,&options);
        self.out_handle.write_fmt(format_args!(
                    "Launching the {}",
                    self.current_list[index].names.ru
                ));
                self.out_handle.flush();
                watch_title(&self.current_list[index]);
                //term.read_key();
                read().unwrap();
        return MenuType::Back;
    }
    fn fetch_latest_menu(&mut self) -> MenuType{
        //let out = io::stdout();
        queue!(
            self.out_handle,
            cursor::MoveTo(0,5),
            terminal::Clear(ClearType::FromCursorDown),
            Print("Fetching recent releases...\n")
        );
        let mut page:u8 = 1;
        match self.fetch_updates_list(page){
            Err(_) => {
                
                //wait for user input
                loop {
                    self.out_handle.execute(cursor::Hide);
                    if read().unwrap() == Event::Key(KeyCode::Enter.into()){
                        break;
                    }    
                }
                return MenuType::Back
            },
            Ok(_)=> return self.list_releases_interact(),

        }
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
        let stdin = stdin();
        queue!(
            self.out_handle,
            cursor::MoveTo(0,5),
            terminal::Clear(terminal::ClearType::FromCursorDown),
    );
        
        self.out_handle.write(b"Enter search name: ");
        self.out_handle.queue(cursor::Show);
        self.out_handle.flush();
        let mut search_name = String::new();
        stdin.read_line(&mut search_name);
        let response = search_title(&search_name);
        let mut jsonVal: Value = serde_json::from_str(response.as_str()).expect("Parsing error");
        self.current_list = serde_json::from_value(jsonVal["list"].take()).expect("parsing error");
        self.out_handle.queue(cursor::Hide);
        self.list_releases_interact();
        return MenuType::Back;
    }
    pub fn list_releases(&mut self) {
        
        for i in 0..self.current_list.len() {
            self.out_handle
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
        self.out_handle.flush();
        io::stdin().read_line(&mut input).expect("input error");
        let index: usize = input.trim().parse().unwrap();
        self.out_handle.write_fmt(format_args!(
            "Launching the {}",
            self.current_list[index].names.en
        ));
        self.out_handle.queue(cursor::Hide);
        self.out_handle.flush();
        watch_title(&self.current_list[index]);

        //return MenuType::Back;
    }
    pub fn list_releases_interact(&mut self)-> MenuType{
        //Printing current list
        let mut selected_option = 1;
        queue!(self.out_handle,cursor::MoveTo(0,5),terminal::Clear(ClearType::FromCursorDown));
        loop{
            for i in 1..self.current_list.len()+1 {
                if i == selected_option {
                    queue!(
                        self.out_handle,
                        SetForegroundColor(Color::Black),
                        SetBackgroundColor(Color::White),
                        Print(format_args!("{}. {}\n", i,self.current_list[i-1].names.ru)),
                        SetForegroundColor(Color::White),
                        SetBackgroundColor(Color::Black),
                    );
                } else {
                    self.out_handle.queue(Print(format_args!("{}. {}\n",i,self.current_list[i-1].names.ru)));
                }
            }
            self.out_handle.queue(Print("0. Next page\n"));
            self.out_handle.queue(cursor::MoveTo(0,5));
            self.out_handle.flush();
            match process_user_interaction(&mut selected_option, &self.current_list) {
                1 =>{
                    //if selected_option == 
                    watch_title(&self.current_list[selected_option-1]);                  
                }
                -1 => {
                    return MenuType::Back;
                }
                _ => {continue;}
            }
        }
    }
}
pub fn search_title(name: &String) -> String {
    
    let searchEndpoint = "https://api.anilibria.tv/v3/title/search";
    return reqwest::blocking::get(format!(
        "{searchEndpoint}?limit=9&order_by=id&search={name}"
    ))
    .expect("msg")
    .text()
    .expect("msg");
}

//##################################################################
fn credentials(std_out:&mut Stdout) {
    //let mut out = io::stdout().lock();
    std_out.write_fmt(format_args!("{:-<52}\n", ""));
    std_out.write_fmt(format_args!("{:<120}\n", "CLI anime episode parser"));
    std_out.write_fmt(format_args!(
        "{:<120}\n",
        "All voiceover rights reserved by Anilibria team"
    ));
    std_out.write_fmt(format_args!(
        "{:<120}\n",
        "Check out their website! https://anilibria.top"
    ));
    std_out.write_fmt(format_args!("{:-<52}\n", ""));
    std_out.flush();
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
fn process_user_interaction(selected_option:&mut usize,list:&Vec<Title>)->i8{
    
    loop {            
        let event = read().unwrap();
        match event {
            Event::Key(event) if event.kind == KeyEventKind::Press => match event.code {
                KeyCode::Esc => {
                    return -1;
                }
                KeyCode::Enter => { return 1}
                KeyCode::Down => {
                    *selected_option += 1;
                    if *selected_option >= list.len() {
                        *selected_option = 0;
                    }
                    return 0;
                }
                KeyCode::Up => {
                    if *selected_option == 0 {
                        *selected_option = list.len();
                    }
                    *selected_option -= 1;
                    return 0;
                }
                KeyCode::Char(c) => {
                    match c.to_digit(10){
                        Some(number)=>{
                            let index = number as usize;
                            if index <= list.len(){
                                *selected_option = index;
                                return 0;
                            }
                        }
                        None=>{
                            continue;
                        }
                    }
                }
                _ => {continue;}
            },
            _ => {}
        }

    }
}