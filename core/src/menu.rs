use crate::app::App;
use crossterm::{
    cursor,
    event::{self, read, Event, KeyCode, KeyEvent, KeyEventKind},
    queue,
    style::Print,
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};
use std::{
    io::{Stdout, Write},
    usize,
};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};

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
    pub show_numbers: bool,
    pub action: fn(&mut App) -> MenuType,
}

//##################################################################
// Provides appropriate logic for each MenuType
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
                action: App::search_prompt,
            };
        }
        MenuType::Back => {
            todo!()
        }
    }
}
// Outdated
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
                    if (pages.0 + 1) > pages.1 {
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
// Main menu drawing loops
pub fn interactive_menu(
    current_list: &Vec<String>,
    app: &mut App,
    menu_name: &str,
    pages: Option<(u32, u32)>,
) -> (UserAction, Option<usize>) {
    let mut state = ListState::default();
    state.select(Some(0));
    let length = current_list.len();
    let pagination_title = pages
        .as_ref()
        .map(|(curr, total)| format!(" {menu_name} | Page: {curr}/{total} "));

    loop {
        app.terminal
            .draw(|f| {
                let size = f.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Min(10), Constraint::Length(1)].as_ref())
                    .split(size);
                let options: Vec<ListItem> = current_list
                    .iter()
                    .map(|i| ListItem::new(i.as_str()))
                    .collect();
                let list = List::new(options)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(pagination_title.as_deref().unwrap_or(menu_name))
                            .title_alignment(Alignment::Center),
                    )
                    .highlight_style(Style::default().fg(Color::Red))
                    .highlight_symbol("\u{25ae} ");
                let hint_list = vec![
                    Span::styled(
                        "Esc",
                        Style::default()
                            .fg(Color::White)
                            .bg(Color::Rgb(150, 0, 0))
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Back | "),
                    Span::styled(
                        "Enter",
                        Style::default()
                            .bg(Color::Rgb(150, 0, 0))
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Select | "),
                    Span::styled(
                        "\u{2191}\u{2193}",
                        Style::default()
                            //.bg(Color::Red)
                            .bg(Color::Rgb(150, 0, 0))
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Navigate options | "),
                    Span::styled(
                        "\u{2190}\u{2192}",
                        Style::default()
                            //.bg(Color::Red)
                            .bg(Color::Rgb(150, 0, 0))
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Navigate pages | "),
                    Span::styled(
                        "[1-9]",
                        Style::default()
                            //.bg(Color::Red)
                            .bg(Color::Rgb(150, 0, 0))
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Select option"),
                ];
                let text = Text::from(Spans::from(hint_list));
                let help_hint = Paragraph::new(text)
                    .block(Block::default().borders(Borders::NONE))
                    .alignment(Alignment::Center);
                f.render_stateful_widget(list, chunks[0], &mut state);
                f.render_widget(help_hint, chunks[1]);
            })
            .expect("unexpected error");

        // Process user response
        if let event::Event::Key(KeyEvent { code, kind, .. }) = event::read().unwrap() {
            if kind == KeyEventKind::Press {
                match code {
                    KeyCode::Up => {
                        if let Some(selected) = state.selected() {
                            if selected > 0 {
                                state.select(Some(selected - 1));
                            } else {
                                state.select(Some(length - 1));
                            }
                        }
                    }
                    KeyCode::Down => {
                        if let Some(selected) = state.selected() {
                            if selected < length - 1 {
                                state.select(Some(selected + 1));
                            } else {
                                state.select(Some(0));
                            }
                        }
                    }
                    KeyCode::Right => match pages {
                        Some((current, total)) => {
                            if current >= total {
                                continue;
                            } else {
                                return (UserAction::PageForward, None);
                            }
                        }
                        None => continue,
                    },
                    KeyCode::Left => match pages {
                        Some((current, _)) => {
                            if current <= 1 {
                                continue;
                            }
                            return (UserAction::PageBackward, None);
                        }
                        None => continue,
                    },
                    KeyCode::Char(c) => match c.to_digit(10) {
                        Some(digit) => {
                            let digit = digit as usize;
                            if digit > 0 && digit <= length {
                                state.select(Some(digit - 1));
                            }
                        }
                        None => todo!(),
                    },

                    KeyCode::Enter => return (UserAction::Select, state.selected()),
                    KeyCode::Esc => return (UserAction::Back, None),
                    _ => {}
                }
            }
        }
    }
}
// Interactive menu for text input
pub fn input_menu(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    msg: &str,
    title: &str,
) -> Option<String> {
    let mut input = String::with_capacity(16);
    loop {
        terminal
            .draw(|f| {
                let size = f.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Min(4), Constraint::Length(1)])
                    .split(size);
                let spans = vec![Span::raw(msg), Span::raw(input.as_str())];
                let text = Paragraph::new(Spans::from(spans)).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(title)
                        .title_alignment(Alignment::Center),
                );
                let hints = vec![
                    Span::styled(
                        "Enter",
                        Style::default()
                            .bg(Color::Rgb(150, 0, 0))
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Accept | "),
                    Span::styled(
                        "Esc",
                        Style::default()
                            .bg(Color::Rgb(150, 0, 0))
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Back"),
                ];
                let hint_text = Paragraph::new(Spans::from(hints))
                    .block(Block::default())
                    .alignment(Alignment::Center);
                f.render_widget(text, chunks[0]);
                f.render_widget(hint_text, chunks[1]);
            })
            .expect("Unexpected error");
        // Process user actions
        if let Event::Key(key) = event::read().expect("Unexpected error") {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char(ch) => {
                        input.push(ch);
                    }
                    KeyCode::Esc => break,
                    KeyCode::Backspace => {
                        input.pop();
                    }
                    KeyCode::Enter => {
                        if input.is_empty() {
                            continue;
                        }

                        return Some(input);
                    }

                    _ => {}
                }
            }
        }
    }
    return None;
}
// Outdated
pub fn read_line_interactive(out_handle: &mut Stdout) -> Option<String> {
    let mut buffer = String::with_capacity(50);
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
