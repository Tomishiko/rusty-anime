// #![allow(dead_code)]
// #![allow(unused_variables)]
// #![allow(unused_imports)]
mod api;
mod app;
mod menu;
use app::{App, Config};
use clap::{arg, ArgMatches, Command};
use crossterm::{execute, terminal};
use menu::*;
use serde::Deserialize;
use std::{
    env, fs,
    io::{self, Read, Write},
    panic,
};
use tui::backend::CrosstermBackend;
use tui::Terminal;
fn main() {
    let exepath = env::current_exe().unwrap();
    env::set_current_dir(exepath.parent().expect("Exe should have parent dir"))
        .expect("Unable to set current dir");

    // Panic hook to log errors in case of unexpected panic
    panic::set_hook(Box::new(|x| {
        let mut file = std::fs::File::create("errorlog.txt").expect("Unable to create log file");
        file.write_fmt(format_args!("{}", x))
            .expect("Unable to wrtie into log file");
    }));

    let config: Config;
    match fs::read_to_string("./config.yaml") {
        Ok(val) => {
            let deserializer = serde_yaml::Deserializer::from_str(&val);
            match Config::deserialize(deserializer) {
                Ok(value) => config = value,
                Err(err) => {
                    println!("Bad config file format!\n{}", err);
                    io::stdin().read(&mut [0u8]).expect("Can not read stdin");
                    return;
                }
            };
        }
        Err(err) => {
            println!(
                "Error trying to read config.yaml: {}\nCheck if the file is present",
                err
            );
            io::stdin().read(&mut [0u8]).expect("Can not read stdin");
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

    let mut stdout = io::stdout();
    // Term setup
    crossterm::terminal::enable_raw_mode().expect("Can't enable raw mode");
    execute!(stdout, terminal::EnterAlternateScreen,).expect("Can not write to stdout");

    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend).expect("Unexpected error happened");
    // app state
    let mut app = App::new(io::stdout(), config, exepath, terminal);

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
    if app.proc.is_some() {
        app.proc
            .unwrap()
            .kill()
            .expect("Unable to kill child process");
    }

    execute!(app.terminal.backend_mut(), terminal::LeaveAlternateScreen)
        .expect("Unable to leave alternate screen");
    crossterm::terminal::disable_raw_mode().expect("Unable to disable raw mode");
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
