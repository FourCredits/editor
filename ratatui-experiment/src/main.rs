use std::{fmt::Display, fs, io, time::Duration};

use anyhow::{Context, Error, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};

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

#[derive(Default, Debug)]
struct App {
    current_file_name: Option<String>,
    open_file_name: Option<String>,
    input_destination: InputDestination,
    file_contents: String,
    messages: Vec<String>,
    message_visible: bool,
}

impl App {
    fn accept_input(&mut self, input: Input) -> bool {
        match input {
            Input::None => (),
            Input::Backspace => self.backspace(),
            Input::Enter => {
                if let Err(err) = self.enter() {
                    self.add_message(err.to_string());
                }
            }
            Input::Cancel => return false,
            Input::NormalChar(c) => self.add_char(c),
            Input::Save => self.input_destination = InputDestination::Save,
            Input::Open => self.input_destination = InputDestination::Open,
            Input::New => self.new_file(),
            Input::ClearMessage => self.clear_message(),
        }
        true
    }

    fn backspace(&mut self) {
        match self.input_destination {
            InputDestination::Buffer => _ = self.file_contents.pop(),
            InputDestination::Open => {
                if let Some(open) = &mut self.open_file_name {
                    _ = open.pop()
                }
            }
            InputDestination::Save => {
                if let Some(current) = &mut self.current_file_name {
                    _ = current.pop()
                }
            }
        }
    }

    fn enter(&mut self) -> Result<(), EditorError> {
        match self.input_destination {
            InputDestination::Buffer => {
                self.file_contents.push('\n');
                Ok(())
            }
            InputDestination::Save => {
                let result = self.save_file();
                self.input_destination = InputDestination::Buffer;
                self.clear_message();
                result
            }
            InputDestination::Open => {
                let result = self.open_file();
                self.input_destination = InputDestination::Buffer;
                self.clear_message();
                result
            }
        }
    }

    fn save_file(&mut self) -> Result<(), EditorError> {
        let path = self
            .current_file_name
            .as_ref()
            .ok_or(EditorError::NoFileSpecified)?;
        fs::write(path, self.file_contents.as_bytes())?;
        Ok(())
    }

    fn open_file(&mut self) -> Result<(), EditorError> {
        let path = self
            .open_file_name
            .take()
            .ok_or(EditorError::NoFileSpecified)?;
        self.file_contents = fs::read_to_string(&path)?;
        self.current_file_name = Some(path);
        Ok(())
    }

    fn add_char(&mut self, c: char) {
        match self.input_destination {
            InputDestination::Buffer => self.file_contents.push(c),
            InputDestination::Open => self.open_file_name.get_or_insert_with(String::new).push(c),
            InputDestination::Save => self
                .current_file_name
                .get_or_insert_with(String::new)
                .push(c),
        }
    }

    fn new_file(&mut self) {
        self.file_contents.clear();
        self.current_file_name = None;
    }

    fn add_message(&mut self, message: String) {
        self.messages.push(message);
        self.message_visible = true;
    }

    fn clear_message(&mut self) {
        self.message_visible = false;
    }
}

#[derive(Debug)]
enum EditorError {
    NoFileSpecified,
    IoError(io::Error),
}

impl From<io::Error> for EditorError {
    fn from(value: io::Error) -> Self {
        Self::IoError(value)
    }
}

impl Display for EditorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditorError::NoFileSpecified => f.write_str("No file specified"),
            EditorError::IoError(err) => {
                f.write_str("IO error: ")?;
                err.fmt(f)
            }
        }
    }
}

impl std::error::Error for EditorError {}

#[derive(Debug, Default, PartialEq, Eq)]
enum InputDestination {
    #[default]
    Buffer,
    Open,
    Save,
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
    if let Some(error) = app.messages.last().filter(|_| app.message_visible) {
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

enum Input {
    None,
    Backspace,
    Enter,
    Cancel,
    NormalChar(char),
    Save,
    Open,
    New,
    ClearMessage,
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
