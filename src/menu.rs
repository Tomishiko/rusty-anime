use crate::api::*;
use crate::app::App;
use console::colors_enabled_stderr;
use console::Term;
use crossterm::event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::{poll, read, Event};
use crossterm::queue;
use crossterm::style::{Color, Print, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{Clear, ClearType};
use crossterm::{
    cursor,
    style::{self, Stylize},
    terminal, ExecutableCommand, QueueableCommand,
};
use reqwest::Client;
use serde::de::value;
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
use std::vec;

//####################################################### Data types

pub enum UserAction {
    Select,
    Back,
    PageForward,
    PageBackward,
}
pub enum MenuType {
    Main,
    Search,
    List,
    Back,
}
pub struct MenuNode {
    //pub parent: MenuNode,
    pub show_numbers: bool,
    pub action: fn(&mut App) -> MenuType,
}

//##################################################################
pub fn credentials(std_out: &mut Stdout) {
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
                show_numbers: false,
                action: App::main_menu,
            };
        }
        MenuType::List => {
            return MenuNode {
                show_numbers: true,
                action: App::fetch_latest_menu,
            };
        }
        MenuType::Search => {
            return MenuNode {
                show_numbers: true,
                action: App::search_logic,
            };
        }
        MenuType::Back => {
            todo!()
        }
    }
}
fn process_user_interaction(
    selected_option: &mut usize,
    list_size: usize,
    pages: (u32, u32),
) -> KeyCode {
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
                    if (pages.0 + 1) >= pages.1 {
                        continue;
                    }
                    return KeyCode::Right;
                }
                KeyCode::Left => {
                    if (pages.0 - 1) <= 0 {
                        continue;
                    }

                    return KeyCode::Left;
                }
                KeyCode::Char(c) => match c.to_digit(10) {
                    Some(number) => {
                        let index = number as usize;
                        if index <= list_size && index != 0 {
                            *selected_option = index - 1;
                            return KeyCode::Null;
                        }
                    }
                    None => {
                        continue;
                    }
                },
                _ => {
                    continue;
                }
            },
            _ => {}
        }
    }
}
pub fn interactive_menu(
    current_list: &Vec<String>,
    app: &mut App,
    menu_type: MenuType,
    pages: Option<(u32, u32)>,
) -> (UserAction, Option<usize>) {
    //Printing current list

    let mut selected_option = 0;
    queue!(
        app.out_handle,
        cursor::Hide,
        cursor::MoveTo(0, 5),
        terminal::Clear(ClearType::FromCursorDown)
    );
    loop {
        for i in 0..current_list.len() {
            if i == selected_option {
                queue!(
                    app.out_handle,
                    SetForegroundColor(Color::Black),
                    SetBackgroundColor(Color::White),
                    Print(&current_list[i]),
                    SetForegroundColor(Color::White),
                    SetBackgroundColor(Color::Black),
                )
                .unwrap();
            } else {
                app.out_handle.queue(Print(&current_list[i]));
            }
        }
        let is_page_displayed = pages.is_some();

        let pages = pages.unwrap_or((1, 1));
        if is_page_displayed {
            app.out_handle
                .queue(Print(format!("\nPage < {} >", pages.0)));
        }

        //app.out_handle.queue(Print("0. Next page\n"));
        app.out_handle.queue(cursor::MoveTo(0, 5));
        app.out_handle.flush();
        match process_user_interaction(&mut selected_option, current_list.len(), pages) {
            KeyCode::Enter => {
                return (UserAction::Select, Some(selected_option));
            }
            KeyCode::Esc => {
                return (UserAction::Back, None);
            }
            KeyCode::Right => {
                return (UserAction::PageForward, None);
            }
            KeyCode::Left => {
                return (UserAction::PageBackward, None);
            }
            _ => {
                continue;
            }
        }
    }
}
pub fn read_line_inter(out_handle: &mut Stdout) -> Option<String> {
    let mut buffer = String::new();
    out_handle.execute(cursor::SavePosition);
    loop {
        let event = read().unwrap();
        match event {
            Event::Key(event) if event.kind == KeyEventKind::Press => match event.code {
                KeyCode::Esc => {
                    return None;
                }
                KeyCode::Enter => {
                    return Some(buffer);
                }
                KeyCode::Char(c) =>
                /*if c.is_digit(10)*/
                {
                    out_handle.queue(Print(c));
                    buffer.push(c);
                }
                KeyCode::Backspace => {
                    // queue!(
                    //     out_handle,
                    //     cursor::MoveLeft(1),
                    //     Print(" "),
                    //     cursor::MoveLeft(1));
                    buffer.pop();
                }
                _ => {
                    continue;
                }
            },
            _ => {}
        }
        queue!(
            out_handle,
            cursor::RestorePosition,
            terminal::Clear(ClearType::UntilNewLine),
            Print(&buffer),
        );

        out_handle.flush();
    }
}
