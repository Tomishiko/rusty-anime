use console::colors_enabled_stderr;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::queue;
use crossterm::event::{poll, read, Event};
use crossterm::style::{Color, Print, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{Clear,ClearType};
use crossterm::{
    cursor,
    style::{self, Stylize},
    terminal, ExecutableCommand, QueueableCommand,
};
use serde::de::value;
use serde_json::Value;
use reqwest::Client;
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
use std::vec;
use crate::api;
use crate::api::*;

//####################################################### Data types

pub enum NavType {
    TypeIn,
    Interactive,
}
pub enum UserAction {
    Select,
    Back,
    PageForward,
    PageBackward
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
    fn fetch_latest_menu(&mut self) -> MenuType{
        queue!(
            self.out_handle,
            cursor::MoveTo(0,5),
            terminal::Clear(ClearType::FromCursorDown),
            Print("Fetching recent releases...\n")
        );

        let mut page:u8 = 1;
        loop{
            match fetch_updates_list(page){
                Err(err) => {
                    
                    self.out_handle.execute(Print(
                        format!(
                            "Failed to fetch, status code {}\n",err.status().unwrap())));
                    //wait for user input
                    loop {
                        self.out_handle.execute(cursor::Hide);
                        if read().unwrap() == Event::Key(KeyCode::Enter.into()){
                            break;
                        }    
                    }
                    return MenuType::Back;
                },
                Ok(val)=> {
                    self.current_list = val;
                    let options = self.build_release_list();
                    let (action,selection) = interactive_menu(&options,self,MenuType::List,Some(page));

                        match action {
                            UserAction::Select => self.watch_title(selection.unwrap()),
                            UserAction::Back => {},
                            UserAction::PageForward => {
                                page+=1;
                                continue;
                            },
                            UserAction::PageBackward => {
                                page-=1;
                                continue;
                            },
                        }
                    
                    
                    return MenuType::Back;
                }

            }

        }
    }
    fn main_menu(&mut self) -> MenuType {
        let (action,index) = interactive_menu(
            &vec![
                String::from_str("Fetch todays\n").unwrap(),
                String::from_str("Search\n").unwrap(),
            ],
            self,
            MenuType::Main,
            None
        );
        match index.unwrap() {
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
        match search_title(&search_name){
            Ok(val) => {self.current_list = val},
            Err(err) =>{
                self.out_handle.execute(Print(format_args!("Failed to fetch, status code {}\n",err.status().unwrap())));
                return MenuType::Back;
            }
        };

        self.out_handle.execute(cursor::Hide);
        let menu_options = self.build_release_list();
        interactive_menu(&menu_options,self,MenuType::List,None);
        return MenuType::Back;
    }
    fn build_release_list(&self)->Vec<String>{
        let mut list:Vec<String> = Vec::with_capacity(self.current_list.len());
        for i in 0..self.current_list.len(){
            list.push(format!(
                "{}. \"{}\" [{}-{}]\n",
                i+1,
                self.current_list[i].names.ru,
                self.current_list[i].player["episodes"]["first"],
                self.current_list[i].player["episodes"]["last"]));
        }
        return list;
    }

    
    
    
    fn watch_title(&mut self,selected_option:usize) {

        let title = &self.current_list[selected_option];
        let mut inputEpisode = String::new();
        queue!(
            self.out_handle,
            cursor::MoveTo(0,5),
            terminal::Clear(ClearType::FromCursorDown)
        );
        loop {
            
            // what the fuck?
            //self.out_handle.write_fmt(format_args!("{esc}[2J{esc}[1;1H", esc = 27 as char));
            queue!(
                self.out_handle,
                cursor::MoveTo(0,5),
                Print(format!(
                    "Watching now: \"{}\"  [{}-{}]\nEnter episode to watch: ",
                    title.names.ru,
                    title.player["episodes"]["first"],
                    title.player["episodes"]["last"],)),
                cursor::Show
            ).unwrap();            
            self.out_handle.flush();
            io::stdin().read_line(&mut inputEpisode);
            //TODO: maybe add fhd/hd parameters parsing
            let mut episode = title.player["list"][inputEpisode.trim()]["hls"]["fhd"]
                .as_str()
                .expect("error parsing json");
            
            let mut url = format!("{API_PLAYER_CACHE}{episode}");
            self.out_handle.execute(cursor::Hide);
            let output = Command::new("player/AniPlayer.exe")
                .arg(url)
                .output()
                .expect("player");
            inputEpisode.clear();
            self.out_handle.execute(cursor::MoveTo(0,5));
        }
    }
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
//
//TODO this goes to menu module
//
fn process_user_interaction(selected_option:&mut usize,list_size:usize,page:u8)->KeyCode{
    
    loop {            
        let event = read().unwrap();
        match event {
            Event::Key(event) if event.kind == KeyEventKind::Press => match event.code {
                KeyCode::Esc => {
                    return KeyCode::Esc;
                }
                KeyCode::Enter => {
                     return KeyCode::Enter;
                    }
                KeyCode::Down => {
                    *selected_option += 1;
                    if *selected_option >= list_size {
                        *selected_option = 0;
                    }
                    return KeyCode::Null;
                }
                KeyCode::Up => {
                    if *selected_option == 0 {
                        *selected_option = list_size;
                    }
                    *selected_option -= 1;
                    return KeyCode::Null;
                }
                KeyCode::Right => {
                    
                    return KeyCode::Right;
                }
                KeyCode::Left => {
                    if (page - 1) <= 0{
                        continue;
                    }
                    
                    return KeyCode::Left;
                }
                KeyCode::Char(c) => {
                    match c.to_digit(10){
                        Some(number)=>{
                            let index = number as usize;
                            if index <= list_size && index !=0{
                                *selected_option = index-1;
                                return KeyCode::Null;
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
pub fn interactive_menu(
    current_list: &Vec<String>,
    app: &mut App,
    menu_type: MenuType,
    page:Option<u8>)-> (UserAction,Option<usize>) {

    //Printing current list
    
    let mut selected_option = 0;
    queue!(app.out_handle,cursor::Hide,cursor::MoveTo(0,5),terminal::Clear(ClearType::FromCursorDown));
    loop{
        for i in 0..current_list.len() {
            if i == selected_option {
                queue!(
                    app.out_handle,
                    SetForegroundColor(Color::Black),
                    SetBackgroundColor(Color::White),
                    Print(&current_list[i]),
                    SetForegroundColor(Color::White),
                    SetBackgroundColor(Color::Black),
                ).unwrap();
                
            } else {
                app.out_handle.queue(Print(&current_list[i]));
            }
        }
        //app.out_handle.queue(Print("0. Next page\n"));
        app.out_handle.queue(cursor::MoveTo(0,5));
        app.out_handle.flush();
        match process_user_interaction(&mut selected_option, current_list.len(),page.unwrap_or(1)) {
            KeyCode::Enter =>{
                return ( UserAction::Select , Some(selected_option) );
            }
            KeyCode::Esc => {
                return (UserAction::Back,None);
            }
            KeyCode::Right =>{
                return (UserAction::PageForward,None);
            }
            KeyCode::Left =>{
                return (UserAction::PageBackward,None);
            }
            _ => {continue;}
        }
    }
}
 