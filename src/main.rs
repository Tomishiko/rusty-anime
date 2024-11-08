// #![allow(dead_code)]
// #![allow(unused_variables)]
// #![allow(unused_imports)]
mod api;
mod app;
mod menu;
use app::App;
use app::Config;
use clap::arg;
use clap::ArgAction;
use clap::ArgMatches;
use clap::Command;
use crossterm::{cursor, execute, queue, style::Print, terminal, ExecutableCommand};
use menu::*;
use serde::de::value;
use serde::Deserialize;
use serde::Deserializer;
use std::env;
use std::fs;
use std::io;
use std::io::Read;
use std::io::Write;
use std::panic;
use std::path::Path;
fn main() {
    let exepath = env::current_exe().unwrap();
    env::set_current_dir(exepath.parent().expect("Exe should have parent dir"));
    panic::set_hook(Box::new(|x| {
        let mut file = std::fs::File::create("crashdump").unwrap();
        file.write_fmt(format_args!("{}", x));
    }));

    let config: Config;
    match fs::read_to_string("./config.yaml") {
        Ok(val) => {
            let deserializer = serde_yaml::Deserializer::from_str(&val);
            match Config::deserialize(deserializer) {
                Ok(value) => config = value,
                Err(err) => {
                    println!("Bad config file format!\n{}", err);
                    io::stdin().read(&mut [0u8]);
                    return;
                }
            };
        }
        Err(err) => {
            println!(
                "Error trying to read config.yaml: {}\nCheck if the file is present",
                err
            );
            io::stdin().read(&mut [0u8]);
            return;
        }
    };
    // clap cli args setup
    let matches = Command::new("AnimeR")
        .version("1.0")
        .about("about")
        .next_line_help(true)
        .arg(arg!(list: -l  "list latest"))
        .arg(arg!(search: -s  <title_name> "search by title name").conflicts_with("list"))
        .get_matches();

    // Term setup
    terminal::enable_raw_mode();
    let mut stdout = io::stdout();
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        cursor::MoveTo(0, 0),
        terminal::DisableLineWrap
    );

    // app state
    let mut app = App::new(io::stdout(), config, exepath);
    app.credentials();

    // cli args action branching
    if env::args().len() > 1 {
        parse_args(&matches, &mut app);
    // main loop
    } else {
        let mut current: MenuNode = menu_provider(MenuType::Main);
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

    //
    execute!(
        stdout,
        terminal::EnableLineWrap,
        terminal::LeaveAlternateScreen
    );
    terminal::disable_raw_mode();
}
fn parse_args(matches: &ArgMatches, app: &mut App) {
    if matches.get_flag("list") {
        app.fetch_latest_menu();
    } else {
        for id in matches.ids() {
            match id.as_str() {
                "search" => {
                    app.search_logic(matches.get_one("search").unwrap());
                }
                _ => {}
            };
        }
    }
}
