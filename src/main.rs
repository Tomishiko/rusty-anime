#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
use crossterm::cursor;
use crossterm::execute;
use crossterm::queue;
use crossterm::terminal;
use crossterm::QueueableCommand;
use menu::menu_provider;
use menu::MenuNode;
use menu::MenuType;
use menu::NavType;
use menu::App;
use reqwest;
use std::env;
use std::env::current_exe;
use std::io;
use std::io::stdout;
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
    execute!(io::stdout(),terminal::EnterAlternateScreen);
    
    //credentials();
    //term.read_char();
    let mut app = App::new();

    let mut current = menu_provider(MenuType::Main);
    //app.menu_stack.push(current);
    //let mut menu_stack:Vec<MenuNode> = Vec::new();
    loop{
        let next = (current.action)(&mut app);
        if matches!(next,MenuType::Back) {
            if(app.menu_stack.len()==0){
                break;
            }
                
            current = app.menu_stack.pop().expect("msg");
        }
        else {
            app.menu_stack.push(current);
            current = menu_provider(next);
                
        }
        
        
    }
    execute!(io::stdout(),terminal::LeaveAlternateScreen);

}
fn navigator() {}
fn search_logic() {}
fn parse_arguments(args: &Vec<String>) {}
