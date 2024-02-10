mod state;

use std::{io, time::Duration};

use anyhow::{Context, Error, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};

use state::{App, Input, InputDestination};

fn main() -> Result<()> {
    setup_panics();
    let result = run();
    restore_terminal().context("failed to restore terminal")?;
    result
}

fn run() -> Result<()> {
    let mut stdout = io::stdout();
    enable_raw_mode().context("failed to enable raw mode")?;
    execute!(stdout, EnterAlternateScreen).context("unable to enter alternate screen")?;
    let mut terminal =
        Terminal::new(CrosstermBackend::new(stdout)).context("creating terminal failed")?;
    let mut app = App::default();
    let mut should_continue = true;
    while should_continue {
        terminal.draw(|frame| render_app(frame, &app))?;
        should_continue = app.accept_input(get_input()?);
    }
    Ok(())
}

fn setup_panics() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        restore_terminal().unwrap();
        original_hook(panic);
    }));
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode().context("failed to disable raw mode")?;
    execute!(io::stdout(), LeaveAlternateScreen).context("unable to switch to main screen")?;
    Ok(())
}

fn render_app(frame: &mut Frame, app: &App) {
    let mut content_box = frame.size();
    let text = paragraph_with_block(
        app.current_file_name.as_deref().unwrap_or("New file"),
        &app.file_contents,
    )
    .wrap(Wrap { trim: true });
    if let Some(dialogue) = get_message(app) {
        let dialogue_box = break_off_top(&mut content_box, 3);
        frame.render_widget(dialogue, dialogue_box);
    }
    if let Some(error) = app.latest_message() {
        let error_box = break_off_top(&mut content_box, 3);
        frame.render_widget(paragraph_with_block("Error", error), error_box);
    }
    frame.render_widget(text, content_box);
}

fn break_off_top(rect: &mut Rect, size: u16) -> Rect {
    let layouts = Layout::new(Direction::Vertical, Constraint::from_mins([size, 0])).split(*rect);
    *rect = layouts[1];
    layouts[0]
}

fn get_message(app: &App) -> Option<Paragraph<'_>> {
    match app.input_destination {
        InputDestination::Buffer => None,
        InputDestination::Open => Some(paragraph_with_block(
            "Open file...",
            app.open_file_name.as_deref().unwrap_or(""),
        )),
        InputDestination::Save => Some(paragraph_with_block(
            "Save as...",
            app.current_file_name.as_deref().unwrap_or(""),
        )),
    }
}

fn paragraph_with_block<'a>(block_title: &'a str, content: &'a str) -> Paragraph<'a> {
    Paragraph::new(content).block(Block::default().borders(Borders::all()).title(block_title))
}

fn get_input() -> Result<Input> {
    if event::poll(Duration::from_millis(250)).context("event poll failed")? {
        if let Event::Key(event) = event::read().context("event read failed")? {
            return get_char(event);
        };
    }
    Ok(Input::None)
}

fn get_char(event: KeyEvent) -> Result<Input> {
    match event {
        KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => Ok(Input::Cancel),
        KeyEvent {
            code: KeyCode::Char('o'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => Ok(Input::Open),
        KeyEvent {
            code: KeyCode::Char('s'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => Ok(Input::Save),
        KeyEvent {
            code: KeyCode::Char('n'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => Ok(Input::New),
        KeyEvent {
            code: KeyCode::Char('x'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => Ok(Input::ClearMessage),
        KeyEvent {
            code: KeyCode::Backspace,
            ..
        } => Ok(Input::Backspace),
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } => Ok(Input::Enter),
        KeyEvent {
            code: KeyCode::Char(c),
            ..
        } => Ok(Input::NormalChar(c)),
        _ => Err(Error::msg("unrecognised key event")),
    }
}
