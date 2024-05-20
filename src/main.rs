#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
use console::style;
use console::Term;
use menu::menu_provider;
use menu::MenuNode;
use menu::MenuType;
use menu::NavType;
use reqwest;
use std::env;
use std::env::current_exe;
use std::io;
use std::io::Read;
use std::io::Write;
use std::option;
use std::ptr::null;
use std::thread;
use std::time::Duration;
fn main() {
    // let args: Vec<String> = env::args().collect();
    // if args.len() != 1{
    //     parse_arguments(&args);
    // }
    let mut term: Term = Term::stdout();
    term.hide_cursor();
    //credentials();
    //term.read_char();
    let mut current = menu_provider(MenuType::Main);
    let mut menu_stack:Vec<MenuNode> = Vec::new();
    loop{
        let next = (current.action)(&term);
        if matches!(next,MenuType::Back) {
            current = menu_stack.pop().expect("msg");
        }
        else {
            menu_stack.push(current);
            current = menu_provider(next);
                
        }
        
        
    }
    term.read_char();
}
fn navigator() {}
fn search_logic() {}
fn parse_arguments(args: &Vec<String>) {}
