use console::Key;
use console::Term;
use std::fmt::format;
use std::io;
use std::io::stdout;
use std::io::Write;
use std::process::Output;
use std::str::FromStr;
use serde::de::value;
use std::ptr::null;
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};
use std::process::Command;



//####################################################### Data types

#[derive(Serialize, Deserialize)]
struct Title {
    names: Name,
    id: i32,
    player:Value,
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
pub enum MenuType {
    Main,
    Search,
    List,
    Back,
}

//##################################################################
fn credentials(){
    // println!("{:^150}","CLI anime episode parser");
    // println!("{:^150}","All voiceover rights reserved by Anilibria team");
    // println!("{:^150}","Check out their website! https://anilibria.top");
    let mut out =  io::stdout().lock();
    out.write_fmt(format_args!("{:-<60}\n",""));
    out.write_fmt(format_args!("{:<120}\n","CLI anime episode parser"));
    out.write_fmt(format_args!("{:<120}\n","All voiceover rights reserved by Anilibria team"));
    out.write_fmt(format_args!("{:<120}\n","Check out their website! https://anilibria.top"));
    out.write_fmt(format_args!("{:-<60}\n",""));
    out.flush();

}

pub fn menu_draw_loop(mut selected_option: usize,mut term: &Term, options: &Vec<String>) -> usize {
    term.clear_screen();
    credentials();
    loop {
        for i in 0..options.len() {
            println!("{}", options[i]);
        }
        term.move_cursor_to(0, 5+selected_option);
        //dbg!(selected_option);
        term.write_fmt(format_args!("{}",">>"));
        match term.read_key() {
            Ok(key) => match key {
                Key::ArrowDown => {
                    selected_option += 1;
                    if selected_option >= options.len() {
                        selected_option = 0;
                    }
                }
                Key::ArrowUp => {
                    if selected_option == 0 {
                        selected_option = options.len();
                    }
                    selected_option -= 1;
                }
                Key::Escape => {
                    break;
                }
                Key::Enter => return selected_option,
                _ => selected_option = 0,
            },
            Err(_) => break,
        }
        term.move_cursor_to(0, 5);
    }
    return selected_option;
}
pub struct MenuNode {
    //pub parent: MenuNode,
    pub show_numbers: bool,
    pub navigation: NavType,
    pub action: fn(term:&Term)-> MenuType,
}

pub fn menu_provider(menu_type: MenuType) -> MenuNode {
    match menu_type {
        MenuType::Main => {
            return MenuNode {
                navigation: NavType::Interactive,
                show_numbers: false,
                action: main_menu,
            };
        }
        MenuType::List => {
            return MenuNode {
                navigation: NavType::TypeIn,
                show_numbers: true,
                action: chose_releases,
            };
        }
        MenuType::Search => {
            return MenuNode {
                navigation: NavType::TypeIn,
                show_numbers: true,
                action:search_logic
            };
        }
        MenuType::Back=>{todo!()}
    }
}

fn fetch_release_list() -> Vec<Title> {
    println!("Fetching releases");
    let resp = reqwest::blocking::get(
        "https://api.anilibria.tv/v3/title/updates?filter=names,player,list,id&since=1715094161&limit=-1",
    )
    .expect("msg")
    .text()
    .expect("msg");

    let mut jsonVal: Value = serde_json::from_str(resp.as_str()).expect("msg");

    let titles: Vec<Title> = serde_json::from_value(jsonVal["list"].take()).expect("parsing error");
    // for i in 0..titles.len(){
    //     println!("{}. {}/{}",i,titles[i].names.ru,titles[i].names.en);
    // }
    return titles;
}

fn chose_releases(term:&Term)->MenuType{
    let titles = fetch_release_list();
    let mut out_handle = io::stdout().lock();
    for i in 0..titles.len(){
        out_handle.write_fmt(format_args!("{}. {}\n",i,titles[i].names.ru)).expect("write error");
    }
    let mut input = String::new();
    io::stdout().write(b"Enter the release number: ").expect("input error");
    term.show_cursor();
    out_handle.flush();
    io::stdin().read_line(&mut input).expect("input error");
    term.hide_cursor();
    let index:usize = input.trim().parse().unwrap();
    out_handle.write_fmt(format_args!("Launching the {}",titles[index].names.en));
    watch_title(&titles[index]);
    term.read_key();
    return MenuType::Back;
}
fn main_menu(term:&Term)->MenuType{
    let index= menu_draw_loop(0, 
                    term, 
                    &vec![String::from_str("Fetch todays").unwrap(),String::from_str("Search").unwrap()]);
    match index {
        0=>{return MenuType::List},
        1=>{return MenuType::Search},
        _=>{return todo!()},
    }
}
fn search_logic(term:&Term)->MenuType{
    println!("Search interface to be implemented");
    term.read_key();
    return MenuType::Back;
}
//TODO:debug interaction logic 
fn watch_title(title:&Title){
    let mut out_handle = io::stdout().lock();
    let mut episode = String::new();
    loop {
        out_handle.write_fmt(format_args!("{esc}[2J{esc}[1;1H", esc = 27 as char));
        out_handle.flush();
        out_handle.write_fmt(format_args!("Enter episode [{}-{}]: ",1,2));
        out_handle.flush();
        io::stdin().read_line(&mut episode);
        let episode = title.player["list"][episode.trim()]["hls"]["hd"].as_str().expect("error parsing json");
        let url =  format!("https://cache.libria.fun{episode}");
        
        let output = Command::new("C:\\Program Files\\KMPlayer 64X\\KMPlayer64.exe")
            .arg(url)
            .output()
            .expect("player");    
    }
    
    
}
