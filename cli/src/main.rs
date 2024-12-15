use commity_lib::*;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Size},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem},
    Terminal, TerminalOptions, Viewport,
};
use std::{
    collections::HashMap,
    io::{self},
    process,
};

const COLOR_HIGHLIGHT: ratatui::prelude::Color = Color::Cyan;
const MARGIN_LEFT: &str = "    ";
const TERMINAL_MIN_HEIGHT: u16 = 7;
const TERMINAL_MIN_WIDTH: u16 = 100;

#[derive(Debug)]
pub enum Symbol {
    Enter,
    Escape,
    Next,
    Previous,
    Page,
    Input,
}

impl Symbol {
    pub fn query(&self, symbols: &TuiSymbols) -> &'static str {
        match self {
            Symbol::Enter => match symbols {
                TuiSymbols::None => "CR",
                TuiSymbols::Unicode => "⏎",
                TuiSymbols::NerdFont => "󰌑",
            },
            Symbol::Escape => match symbols {
                TuiSymbols::None => "ESC",
                TuiSymbols::Unicode => "␛",
                TuiSymbols::NerdFont => "󱊷",
            },
            Symbol::Next => match symbols {
                TuiSymbols::None => "RIGHT",
                TuiSymbols::Unicode => "→",
                TuiSymbols::NerdFont => "",
            },
            Symbol::Previous => match symbols {
                TuiSymbols::None => "LEFT",
                TuiSymbols::Unicode => "←",
                TuiSymbols::NerdFont => "",
            },
            Symbol::Page => match symbols {
                TuiSymbols::None => "Page",
                TuiSymbols::Unicode => "Page",
                TuiSymbols::NerdFont => "",
            },
            Symbol::Input => match symbols {
                TuiSymbols::None => ">",
                TuiSymbols::Unicode => ">",
                TuiSymbols::NerdFont => "",
            },
        }
    }
}

pub enum InputResult {
    Ok(),
    Back,
    Quit,
}

fn pad_to_length(input: &String, total_length: usize) -> String {
    format!("{:<width$}", input, width = total_length)
}

fn draw_frame(
    tui_config: &TUIConfig,
    page: usize,
    pages: usize,
    body_height: u16,
    title: &String,
    description: &String,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    body: List<'_>,
    body_dimensions: Size,
) -> Result<(), Box<dyn std::error::Error>> {
    terminal.draw(|frame| {
        let terminal_area = frame.area();
        let mut render_area = terminal_area;

        if tui_config.layout == TUILayout::FullscreenCenter {
            let content_width = TERMINAL_MIN_WIDTH
                .max(body_dimensions.width)
                .min(terminal_area.width - 8);

            let content_height = (body_height + TERMINAL_MIN_HEIGHT).min(terminal_area.height);
            let offset_x = (terminal_area.width - content_width) / 2;
            let offset_y = (terminal_area.height - content_height) / 2;

            render_area = ratatui::layout::Rect {
                x: offset_x,
                y: offset_y,
                width: content_width,
                height: content_height,
            };
        }

        let flex = match tui_config.layout {
            TUILayout::Inline => ratatui::layout::Flex::Start,
            TUILayout::Fullscreen => ratatui::layout::Flex::Start,
            TUILayout::FullscreenCenter => ratatui::layout::Flex::Center,
        };

        let chunks = Layout::default()
            .flex(flex)
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(2),
                match tui_config.layout {
                    TUILayout::Inline => Constraint::Length(body_height),
                    TUILayout::Fullscreen => Constraint::Min(body_height),
                    TUILayout::FullscreenCenter => Constraint::Length(body_height),
                },
                Constraint::Length(1),
                Constraint::Length(2),
            ])
            .split(render_area);

        let title = Line::from(vec![Span::styled(
            format!("{}{}", MARGIN_LEFT, &title),
            Style::default(),
        )]);
        frame.render_widget(title, chunks[1]);

        let description = Line::from(vec![Span::styled(
            format!("{}{}", MARGIN_LEFT, &description),
            Style::default().add_modifier(Modifier::DIM),
        )]);
        frame.render_widget(description, chunks[2]);

        //frame.render_widget(list, area);
        frame.render_widget(body, chunks[3]);

        let footer = Line::from(vec![
            Span::styled(
                format!(
                    "{}{}",
                    MARGIN_LEFT,
                    format!(
                        "{} {}/{}",
                        Symbol::Page.query(&tui_config.symbols),
                        page,
                        pages
                    )
                ),
                Style::default().add_modifier(Modifier::DIM),
            ),
            Span::styled(
                format!(" • [{}] Quit", Symbol::Escape.query(&tui_config.symbols)),
                Style::default().add_modifier(Modifier::DIM),
            ),
            if page != 1 {
                Span::styled(
                    format!(" • [{}] Back", Symbol::Previous.query(&tui_config.symbols)),
                    Style::default().add_modifier(Modifier::DIM),
                )
            } else {
                Span::raw("")
            },
            Span::styled(format!(" • "), Style::default().add_modifier(Modifier::DIM)),
            if page == pages {
                Span::styled(
                    format!("[{}] Commit", Symbol::Enter.query(&tui_config.symbols)),
                    Style::default(),
                )
            } else {
                Span::styled(
                    format!("[{}] Accept", Symbol::Enter.query(&tui_config.symbols)),
                    Style::default().add_modifier(Modifier::DIM),
                )
            },
        ]);
        frame.render_widget(footer, chunks[5]);
    })?;
    Ok(())
}

fn input_choice(
    tui_config: &TUIConfig,
    page: usize,
    pages: usize,
    entry_choice: &mut EntryChoice,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<InputResult, Box<dyn std::error::Error>> {
    let mut selected_index = entry_choice
        .choices
        .iter()
        .position(|choice| choice.value == entry_choice.value)
        .unwrap_or(0); // Initialize with current value
    let width_nr = entry_choice.choices.len().to_string().len();
    let width_val = entry_choice
        .choices
        .iter()
        .map(|choice| choice.value.len())
        .max()
        .unwrap_or(0);

    let viewport_height = terminal.size()?.height.saturating_sub(TERMINAL_MIN_HEIGHT) as usize; // Leave space for header/footer
    let mut viewport_start = 0; // Start index of the visible viewport
    let mut viewport_end = viewport_height.min(entry_choice.choices.len()); // End index of the visible viewport

    loop {
        if selected_index < viewport_start {
            viewport_start = selected_index;
            viewport_end = (viewport_start + viewport_height).min(entry_choice.choices.len());
        } else if selected_index >= viewport_end {
            viewport_end = selected_index + 1;
            viewport_start = viewport_end.saturating_sub(viewport_height);
        }

        let mut max_width = 0;
        let mut max_height = 0;

        let items: Vec<ListItem> = entry_choice
            .choices
            .iter()
            .enumerate()
            .skip(viewport_start)
            .map(|(i, choice)| {
                let content = if i == selected_index {
                    let line = Line::from(vec![
                        Span::styled(
                            format!(
                                "{}● {} {}    ",
                                MARGIN_LEFT,
                                pad_to_length(&format!("{}.", (i + 1).to_string()), width_nr),
                                pad_to_length(&choice.value, width_val),
                            ),
                            Style::default().fg(COLOR_HIGHLIGHT),
                        ),
                        Span::styled(
                            format!("{}", &choice.label),
                            Style::default().add_modifier(Modifier::DIM),
                        ),
                    ]);
                    max_width = line.width().max(max_width);
                    max_height += 1;
                    line
                } else {
                    let line = Line::from(vec![
                        Span::styled(
                            format!(
                                "{}○ {} {}    ",
                                MARGIN_LEFT,
                                pad_to_length(&format!("{}.", (i + 1).to_string()), width_nr),
                                pad_to_length(&choice.value, width_val),
                            ),
                            Style::default(),
                        ),
                        Span::styled(
                            format!("{}", &choice.label),
                            Style::default().add_modifier(Modifier::DIM),
                        ),
                    ]);
                    max_width = line.width().max(max_width);
                    max_height += 1;
                    line
                };
                ListItem::new(content)
            })
            .collect();

        let body = List::new(items);

        draw_frame(
            tui_config,
            page,
            pages,
            entry_choice.choices.len() as u16,
            &entry_choice.label,
            &entry_choice.description,
            terminal,
            body,
            Size::new(max_width as u16, max_height as u16),
        )?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up => {
                    if selected_index > 0 {
                        selected_index -= 1;
                    }
                }
                KeyCode::Down => {
                    if selected_index < entry_choice.choices.len() - 1 {
                        selected_index += 1;
                    }
                }
                KeyCode::Left => {
                    return Ok(InputResult::Back);
                }
                KeyCode::Esc => {
                    return Ok(InputResult::Quit);
                }
                KeyCode::Enter => {
                    entry_choice.value = entry_choice.choices[selected_index].value.clone(); // Update the value
                    return Ok(InputResult::Ok());
                }
                _ => {}
            }
        }
    }
}

fn input_bool(
    tui_config: &TUIConfig,
    page: usize,
    pages: usize,
    entry_boolean: &mut EntryBoolean,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<InputResult, Box<dyn std::error::Error>> {
    let mut selected_index = if entry_boolean.value { 0 } else { 1 }; // Initialize with current value

    loop {
        let items: Vec<ListItem> = vec!["Yes", "No"]
            .iter()
            .enumerate()
            .map(|(i, option)| {
                let content = if i == selected_index {
                    Line::from(vec![Span::styled(
                        format!("{}● {}", MARGIN_LEFT, option),
                        Style::default().fg(Color::Yellow),
                    )])
                } else {
                    Line::from(vec![Span::styled(
                        format!("{}○ {}", MARGIN_LEFT, option),
                        Style::default(),
                    )])
                };
                ListItem::new(content)
            })
            .collect();

        let body = List::new(items);

        draw_frame(
            tui_config,
            page,
            pages,
            2,
            &entry_boolean.label,
            &entry_boolean.description,
            terminal,
            body,
            Size::new(6, 2),
        )?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up | KeyCode::Down => {
                    selected_index = 1 - selected_index; // Toggle
                }
                KeyCode::Left => {
                    return Ok(InputResult::Back);
                }
                KeyCode::Esc => {
                    return Ok(InputResult::Quit);
                }
                KeyCode::Enter => {
                    entry_boolean.value = selected_index == 0; // Update the value
                    return Ok(InputResult::Ok());
                }
                _ => {}
            }
        }
    }
}

fn input_text(
    tui_config: &TUIConfig,
    page: usize,
    pages: usize,
    entry_text: &mut EntryText,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<InputResult, Box<dyn std::error::Error>> {
    let mut input = entry_text.value.clone(); // Initialize with current value

    loop {
        let body_str = format!(
            "{}{} {}",
            MARGIN_LEFT,
            Symbol::Input.query(&tui_config.symbols),
            input
        );
        let body_len = body_str.len() as u16;
        let body = List::new(vec![ListItem::new(Span::raw(body_str))]);

        draw_frame(
            tui_config,
            page,
            pages,
            1,
            &entry_text.label,
            &entry_text.description,
            terminal,
            body,
            Size::new(body_len as u16, 2),
        )?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Left => {
                    return Ok(InputResult::Back);
                }
                KeyCode::Esc => {
                    return Ok(InputResult::Quit);
                }
                KeyCode::Enter => {
                    entry_text.value = input;
                    return Ok(InputResult::Ok());
                }
                KeyCode::Char(c) => {
                    input.push(c);
                }
                _ => {}
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    let mut config = Config::load(&current_dir)?;
    let tui_config = TUIConfig::load()?;

    // find out maximum height
    let mut max_height: u16 = 0;
    let mut pages = 0;
    for section in &config.sections {
        for entry in &section.entries {
            pages += 1;
            match entry {
                Entry::Text(_) => {}
                Entry::Choice(choice) => {
                    let choices = choice.choices.len() as u16;
                    if choices > max_height {
                        max_height = choices;
                    }
                }
                Entry::Boolean(_) => {}
            }
        }
    }
    max_height += TERMINAL_MIN_HEIGHT;

    let mut page = 1; // Current page index
    let mut section_index = 0; // Index of the current section
    let mut entry_index = 0; // Index of the current entry
    let total_sections = config.sections.len(); // Total number of sections

    enable_raw_mode()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: match tui_config.layout {
                TUILayout::Inline => Viewport::Inline(max_height),
                TUILayout::Fullscreen => Viewport::Fullscreen,
                TUILayout::FullscreenCenter => Viewport::Fullscreen,
            },
        },
    )?;
    terminal.clear()?;

    // Main navigation loop
    while section_index < total_sections {
        let section_entries_len = config.sections[section_index].entries.len(); // Total entries in current section
        let section = &mut config.sections[section_index]; // Mutable reference to section

        while entry_index < section_entries_len {
            let entry = &mut section.entries[entry_index]; // Mutable reference to entry

            let result = match entry {
                Entry::Text(entry_text) => {
                    input_text(&tui_config, page, pages, entry_text, &mut terminal)?
                }
                Entry::Choice(entry_choice) => {
                    input_choice(&tui_config, page, pages, entry_choice, &mut terminal)?
                }
                Entry::Boolean(entry_boolean) => {
                    input_bool(&tui_config, page, pages, entry_boolean, &mut terminal)?
                }
            };

            match result {
                InputResult::Ok() => {
                    entry_index += 1; // Move to the next entry
                    page += 1; // Update page number
                }
                InputResult::Back => {
                    if entry_index > 0 {
                        entry_index -= 1; // Move to the previous entry
                    } else if section_index > 0 {
                        section_index -= 1; // Move to the previous section
                        entry_index = section_entries_len - 1; // Last entry of the previous section
                    }
                    if page > 1 {
                        page -= 1;
                    }
                }
                InputResult::Quit => {
                    disable_raw_mode()?;
                    terminal.clear()?;
                    if tui_config.layout != TUILayout::Inline {
                        ratatui::restore();
                    }
                    println!("Commit canceled");
                    process::exit(1);
                }
            }
        }

        // Move to the next section once all entries are processed
        section_index += 1;
        entry_index = 0; // Reset entry index for the next section
    }

    // Exit raw mode and render the final message on successful completion
    disable_raw_mode()?;
    terminal.clear()?;
    if tui_config.layout != TUILayout::Inline {
        ratatui::restore();
    }
    println!(
        "Committed --->{}<---",
        render_message(
            config
                .sections
                .iter()
                .flat_map(|section| &section.entries)
                .filter_map(|entry| match entry {
                    Entry::Text(entry_text) => Some((
                        entry_text.name.clone(),
                        FieldValue::Text(entry_text.value.clone())
                    )),
                    Entry::Choice(entry_choice) => Some((
                        entry_choice.name.clone(),
                        FieldValue::Text(entry_choice.value.clone())
                    )),
                    Entry::Boolean(entry_boolean) => Some((
                        entry_boolean.name.clone(),
                        FieldValue::Boolean(entry_boolean.value)
                    )),
                })
                .collect::<HashMap<_, _>>(),
            &config.template
        )?
    );

    Ok(())
}
