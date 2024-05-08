#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
use std::env;
use std::io;
use std::io::Read;
use std::io::Write;
use std::option;
use reqwest;
use std::thread;
use std::time::Duration;
use console::Term;
use console::Key;
use console::style;
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};

//Data types
#[derive(Serialize,Deserialize)]
struct Title {
    names: name,
    id:i32,

}
#[derive(Serialize,Deserialize)]
struct name{
    ru:String,
    en:String,
}
#[derive(Serialize, Deserialize)]
struct  Response {
    list:Vec<Title>
    
}
//##################################################################
fn main(){
    // let args: Vec<String> = env::args().collect();
    // if args.len() != 1{
    //     parse_arguments(&args);
    // }
    let term = Term::stdout();
    term.hide_cursor();


    let init_menu = ["Todays releases","Search release"];

    let selected = menu_draw_loop(0,&term,&init_menu);
    match selected {
        0=>{
            fetch_release_list();
        },
        _=>{}
    }
    term.read_char();
}

fn menu_draw_loop(mut selected_option:usize,term:&Term,options:&[&str])->usize{
    term.clear_screen();
    loop{
        for i in 0..options.len(){
            println!("   {}",options[i]);
        }
        term.move_cursor_to(0, selected_option);
        //dbg!(selected_option);
        term.write_line(">>");
        match term.read_key()
        {
            Ok(key)=> match key{
                Key::ArrowDown=>{ 
                        selected_option+=1;
                        if selected_option >= options.len(){
                            selected_option=0;
                        }
                    },
                Key::ArrowUp=>{
                    if selected_option == 0{
                        selected_option=options.len();
                    }
                    selected_option-=1;
                    },
                Key::Escape => { break;},
                Key::Enter => { return selected_option},
                _ =>selected_option = 0,
            },
            Err(_)=>break,
        }
        term.move_cursor_to(0, 0);
    }
    return selected_option;
}

fn navigator(){

}
fn init_menu()->u8{
    println!("1. Todays releases\n2. Search\nEnter desired number...");
    let mut input = [0u8];
    match io::stdin().read_exact(&mut input){
        Ok(res) => res,
        Err(_) => println!("Wrong menu item"),
    };
    return input[0];
}
fn search_logic(){
    
}
fn parse_arguments(args: &Vec<String>) {
    
}
fn fetch_release_list()->Vec<Title>{
    println!("Fetching releases");
    let resp = 
        reqwest::blocking::get("https://api.anilibria.tv/v3/title/updates?filter=names,id&since=1715094161&limit=-1")
            .expect("msg").text().expect("msg");
    
    let mut jsonVal: Value = serde_json::from_str(resp.as_str()).expect("msg");
    
    let titles:Vec<Title> = serde_json::from_value(jsonVal["list"].take()).expect("parsing error");
    // for i in 0..titles.len(){
    //     println!("{}. {}/{}",i,titles[i].names.ru,titles[i].names.en);
    // }
    return titles;
    
}
