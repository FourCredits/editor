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

fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    let mut contents = String::new();
    loop {
        terminal.draw(|frame| render_app(frame, &contents))?;
        match handle_input()? {
            Some(Some(c)) => {
                if c == '\u{0008}' {
                    _ = contents.pop();
                } else {
                    contents.push(c)
                }
            }
            Some(None) => break,
            None => (),
        }
    }
    Ok(())
}

fn render_app(frame: &mut Frame<CrosstermBackend<Stdout>>, contents: &str) {
    let text = Paragraph::new(contents);
    frame.render_widget(text, frame.size());
}

fn handle_input() -> Result<Option<Option<char>>> {
    if event::poll(Duration::from_millis(250)).context("event poll failed")? {
        if let Event::Key(event) = event::read().context("event read failed")? {
            return Ok(Some(get_char(event)?));
        };
    }
    Ok(None)
}

fn get_char(event: KeyEvent) -> Result<Option<char>> {
    match event {
        KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: _,
            state: _,
        } => Ok(None),
        KeyEvent {
            code: KeyCode::Backspace,
            modifiers: _,
            kind: _,
            state: _,
        } => Ok(Some('\u{0008}')),
        KeyEvent {
            code: KeyCode::Enter,
            modifiers: _,
            kind: _,
            state: _,
        } => Ok(Some('\n')),
        KeyEvent {
            code: KeyCode::Char(c),
            modifiers: _,
            kind: _,
            state: _,
        } => Ok(Some(c)),
        _ => Err(Error::msg("unrecognised key event")),
    }
}
