use color_eyre::section;
use commity_lib::*;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph},
    Terminal, TerminalOptions, Viewport,
};
use std::{
    collections::{hash_map, HashMap},
    io::{self},
};

const COLOR_HIGHLIGHT: ratatui::prelude::Color = Color::Cyan;
const MARGIN_LEFT: &str = "    ";

fn pad_to_length(input: &String, total_length: usize) -> String {
    format!("{:<width$}", input, width = total_length)
}

fn draw_frame(
    page: usize,
    pages: usize,
    title: &String,
    description: &String,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    body: List<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    terminal.draw(|frame| {
        //let area = frame.area();
        let chunks = Layout::default()
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(2),
                Constraint::Min(1),
                Constraint::Length(1),
                Constraint::Length(2),
            ])
            .split(frame.area());

        let title = Line::from(vec![
            Span::styled(
                format!("{}{}/{} ", MARGIN_LEFT, page, pages),
                Style::default().fg(COLOR_HIGHLIGHT),
            ),
            Span::styled(
                format!("{}", &title),
                Style::default().add_modifier(Modifier::DIM),
            ),
        ]);
        frame.render_widget(title, chunks[1]);

        let description = Line::from(vec![Span::styled(
            format!("{}{}", MARGIN_LEFT, &description),
            Style::default().add_modifier(Modifier::DIM),
        )]);
        frame.render_widget(description, chunks[2]);

        //frame.render_widget(list, area);
        frame.render_widget(body, chunks[3]);

        let footer = Line::from(vec![Span::styled(
            format!("{}Im a happy bird", MARGIN_LEFT),
            Style::default().add_modifier(Modifier::DIM),
        )]);
        frame.render_widget(footer, chunks[5]);
    })?;
    Ok(())
}

fn input_choice(
    page: usize,
    pages: usize,
    entry_choice: &EntryChoice,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut selected_index = 0;
    let width_nr = entry_choice.choices.len().to_string().len();
    let width_val = entry_choice
        .choices
        .iter()
        .map(|choice| choice.value.len())
        .max()
        .unwrap_or(0);

    loop {
        let items: Vec<ListItem> = entry_choice
            .choices
            .iter()
            .enumerate()
            .map(|(i, choice)| {
                let content = if i == selected_index {
                    Line::from(vec![
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
                    ])
                } else {
                    Line::from(vec![
                        Span::styled(
                            format!(
                                "{}○ {} {}    ",
                                MARGIN_LEFT,
                                pad_to_length(&format!("{}.", (i + 1).to_string()), width_nr),
                                pad_to_length(&choice.value, width_val),
                            ),
                            Style::default().add_modifier(Modifier::DIM),
                        ),
                        Span::styled(
                            format!("{}", &choice.label),
                            Style::default().add_modifier(Modifier::DIM),
                        ),
                    ])
                };
                ListItem::new(content)
            })
            .collect();

        let body = List::new(items);

        draw_frame(
            page,
            pages,
            &entry_choice.label,
            &entry_choice.description,
            terminal,
            body,
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
                KeyCode::Enter => {
                    return Ok(entry_choice.choices[selected_index].value.clone());
                }
                KeyCode::Esc => {
                    return Err("Selection cancelled".into());
                }
                _ => {}
            }
        }
    }
}

fn input_bool(
    page: usize,
    pages: usize,
    entry_boolean: &EntryBoolean,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut selected_index = if entry_boolean.default { 0 } else { 1 };
    let options = vec!["Yes", "No"];

    loop {
        let items: Vec<ListItem> = options
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
                        Style::default().add_modifier(Modifier::DIM),
                    )])
                };
                ListItem::new(content)
            })
            .collect();

        let body = List::new(items);

        draw_frame(
            page,
            pages,
            &entry_boolean.label,
            &entry_boolean.description,
            terminal,
            body,
        )?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up | KeyCode::Down => {
                    selected_index = 1 - selected_index; // Toggle between 0 and 1
                }
                KeyCode::Enter => {
                    return Ok(selected_index == 0);
                }
                KeyCode::Esc => {
                    return Err("Selection cancelled".into());
                }
                _ => {}
            }
        }
    }
}

fn input_text(
    page: usize,
    pages: usize,
    entry_text: &EntryText,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut input = String::new();

    loop {
        let body = List::new(vec![ListItem::new(Span::raw(format!(
            "{}> {}",
            MARGIN_LEFT, input
        )))]);

        draw_frame(
            page,
            pages,
            &entry_text.label,
            &entry_text.description,
            terminal,
            body,
        )?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Enter => {
                    return Ok(input);
                }
                KeyCode::Char(c) => {
                    input.push(c);
                }
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Esc => {
                    return Err("Text input cancelled".into());
                }
                _ => {}
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the current working directory
    let current_dir = std::env::current_dir()?;

    // Create a Config instance
    let config = Config::new(&current_dir)?;

    // find out maximum height
    let mut max_height = 0;
    let mut pages = 0;
    for section in &config.sections {
        for entry in &section.entries {
            pages += 1;
            match entry {
                Entry::Text(_) => {}
                Entry::Choice(choice) => {
                    if choice.choices.len() > max_height {
                        max_height = choice.choices.len()
                    }
                }
                Entry::Boolean(_) => {}
            }
        }
    }
    max_height += 7; // add 3 extra lines for both header and footer

    enable_raw_mode()?;
    let stdout = io::stdout();

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Inline(max_height as u16),
        },
    )?;

    let mut data: HashMap<String, FieldValue> = HashMap::new();
    let mut page = 0;
    for section in &config.sections {
        for entry in &section.entries {
            page += 1;
            match entry {
                Entry::Text(entry_text) => {
                    let value = input_text(page, pages, entry_text, &mut terminal)?;
                    data.insert(entry_text.name.clone(), FieldValue::Text(value));
                }
                Entry::Choice(entry_choice) => {
                    let value = input_choice(page, pages, entry_choice, &mut terminal)?;
                    data.insert(entry_choice.name.clone(), FieldValue::Text(value));
                }
                Entry::Boolean(entry_boolean) => {
                    let value = input_bool(page, pages, entry_boolean, &mut terminal)?;
                    data.insert(entry_boolean.name.clone(), FieldValue::Boolean(value));
                }
            }
        }
    }

    disable_raw_mode()?;
    terminal.clear()?;
    println!("--->{}<---", render_fields(data, &config.template)?);
    Ok(())
}
