use std::{
    io::{self, Stdout},
    time::Duration,
};

use anyhow::{Context, Error, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};

fn main() -> Result<()> {
    let mut terminal = setup_terminal().context("setup failed")?;
    run(&mut terminal).context("app loop failed")?;
    restore_terminal(&mut terminal).context("restore terminal failed")?;
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = io::stdout();
    enable_raw_mode().context("failed to enable raw mode")?;
    execute!(stdout, EnterAlternateScreen).context("unable to enter alternate screen")?;
    Terminal::new(CrosstermBackend::new(stdout)).context("creating terminal failed")
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode().context("failed to disable raw mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .context("unable to switch to main screen")?;
    terminal.show_cursor().context("unable to show cursor")
}

#[derive(Default, Debug)]
struct App {
    file_contents: String,
}

fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    let mut app = App::default();
    loop {
        terminal.draw(|frame| render_app(frame, &app))?;
        match handle_input()? {
            Input::None => (),
            Input::Backspace => _ = app.file_contents.pop(),
            Input::Cancel => break,
            Input::NormalChar(c) => app.file_contents.push(c),
        }
    }
    Ok(())
}

fn render_app(frame: &mut Frame<CrosstermBackend<Stdout>>, app: &App) {
    let text = Paragraph::new(app.file_contents.clone())
        .block(Block::default().borders(Borders::all()))
        .wrap(Wrap { trim: true });
    frame.render_widget(text, frame.size());
}

enum Input {
    None,
    Backspace,
    Cancel,
    NormalChar(char),
}

fn handle_input() -> Result<Input> {
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
            code: KeyCode::Backspace,
            ..
        } => Ok(Input::Backspace),
        KeyEvent {
            code: KeyCode::Enter,
            ..
        } => Ok(Input::NormalChar('\n')),
        KeyEvent {
            code: KeyCode::Char(c),
            ..
        } => Ok(Input::NormalChar(c)),
        _ => Err(Error::msg("unrecognised key event")),
    }
}
