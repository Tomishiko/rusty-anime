// #![allow(dead_code)]
// #![allow(unused_variables)]
// #![allow(unused_imports)]
mod api;
mod app;
mod menu;
use app::App;
use menu::*;
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
    // execute!(io::stdout(),
    //         terminal::EnterAlternateScreen,
    //         terminal::DisableLineWrap);
    //terminal::enable_raw_mode();

    let mut app = App::new();

    let mut current = menu_provider(MenuType::Main);
    loop {
        let next = (current.action)(&mut app);
        if matches!(next, MenuType::Back) {
            if app.menu_stack.len() == 0 {
                break;
            }

            current = app.menu_stack.pop().expect("msg");
        } else {
            app.menu_stack.push(current);
            current = menu_provider(next);
        }
    }
}
fn navigator() {}
fn search_logic() {}
fn parse_arguments(args: &Vec<String>) {}
