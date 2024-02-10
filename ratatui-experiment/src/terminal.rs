use std::{
    io::{self, Stdout},
    time::Duration,
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::{Constraint, CrosstermBackend, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};

use crate::{
    state::{App, Input, InputDestination},
    ui::Ui,
};

pub struct TerminalUi {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Ui for TerminalUi {
    type Error = io::Error;

    fn on_panic() {
        restore_terminal().unwrap();
    }

    fn setup() -> Result<Self, Self::Error> {
        let mut stdout = io::stdout();
        enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen)?;
        let terminal = Terminal::new(CrosstermBackend::new(stdout))?;
        Ok(Self { terminal })
    }

    fn render(&mut self, app: &App) -> Result<(), Self::Error> {
        self.terminal.draw(|frame| render_app(frame, app))?;
        Ok(())
    }

    fn get_input(&mut self) -> Result<Input, Self::Error> {
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(event) = event::read()? {
                return get_char(event);
            };
        }
        Ok(Input::None)
    }

    fn finish(&mut self) -> Result<(), Self::Error> {
        restore_terminal()
    }
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
    if let Some((label, input)) = get_message(app) {
        let dialogue_box = break_off_top(&mut content_box, 3);
        frame.render_widget(paragraph_with_block(label, input), dialogue_box);
        // adding one because of block borders
        frame.set_cursor(dialogue_box.x + input.len() as u16 + 1, dialogue_box.y + 1)
    } else {
        frame.set_cursor(
            content_box.x + app.file_contents.len() as u16 + 1,
            content_box.y + 1,
        )
    }
    frame.render_widget(text, content_box);
}

fn break_off_top(rect: &mut Rect, size: u16) -> Rect {
    let layouts = Layout::new(Direction::Vertical, Constraint::from_mins([size, 0])).split(*rect);
    *rect = layouts[1];
    layouts[0]
}

fn paragraph_with_block<'a>(block_title: &'a str, content: &'a str) -> Paragraph<'a> {
    Paragraph::new(content).block(Block::default().borders(Borders::all()).title(block_title))
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

fn get_char(event: KeyEvent) -> io::Result<Input> {
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
        _ => todo!("replace this"),
        // _ => Err(Error::msg("unrecognised key event")),
    }
}

fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
