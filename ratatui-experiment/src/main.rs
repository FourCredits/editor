use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
    time::Duration,
};
use std::io::Stdout;

use anyhow::{Context, Error, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};

fn main() -> Result<()> {
    setup_panics();
    let mut terminal = setup_terminal().context("setup failed")?;
    run(&mut terminal).context("app loop failed")?;
    restore_terminal().context("restore terminal failed")?;
    Ok(())
}

fn setup_panics() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        restore_terminal().unwrap();
        original_hook(panic);
    }));
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = io::stdout();
    enable_raw_mode().context("failed to enable raw mode")?;
    execute!(stdout, EnterAlternateScreen).context("unable to enter alternate screen")?;
    Terminal::new(CrosstermBackend::new(stdout)).context("creating terminal failed")
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
}

impl App {
    fn accept_input(&mut self, input: Input) -> bool {
        match input {
            Input::None => (),
            Input::Backspace => self.backspace(),
            Input::Enter => self.enter(),
            Input::Cancel => return false,
            Input::NormalChar(c) => self.add_char(c),
            Input::Save => self.input_destination = InputDestination::Save,
            Input::Open => self.input_destination = InputDestination::Open,
            Input::New => self.new_file(),
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

    fn enter(&mut self) {
        match self.input_destination {
            InputDestination::Buffer => self.file_contents.push('\n'),
            InputDestination::Save => {
                // TODO: get rid of all these unwraps
                let path = Path::new(self.current_file_name.as_ref().unwrap());
                let mut file = File::create(path).unwrap();
                file.write_all(self.file_contents.as_bytes()).unwrap();
                self.input_destination = InputDestination::Buffer;
            }
            InputDestination::Open => {
                // TODO: get rid of all these unwraps
                let path = self.open_file_name.as_ref().unwrap();
                self.file_contents = fs::read_to_string(path).unwrap();
                self.current_file_name = self.open_file_name.take();
                self.input_destination = InputDestination::Buffer;
            }
        }
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
}

#[derive(Debug, Default, PartialEq, Eq)]
enum InputDestination {
    #[default]
    Buffer,
    Open,
    Save,
}

fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    let mut app = App::default();
    let mut should_continue = true;
    while should_continue {
        terminal.draw(|frame| render_app(frame, &app))?;
        should_continue = app.accept_input(get_input()?);
    }
    Ok(())
}

fn render_app(frame: &mut Frame, app: &App) {
    let size = frame.size();
    let text = Paragraph::new(app.file_contents.clone())
        .block(
            Block::default()
                .borders(Borders::all())
                .clone()
                .title(app.current_file_name.as_deref().unwrap_or("New file")),
        )
        .wrap(Wrap { trim: true });
    if let Some(message) = get_message(app) {
        let layout = Layout::new(Direction::Vertical, Constraint::from_mins([3, 0])).split(size);
        frame.render_widget(
            Paragraph::new(message).block(Block::default().borders(Borders::all())),
            layout[0],
        );
        frame.render_widget(text, layout[1]);
    } else {
        frame.render_widget(text, size);
    }
}

fn get_message(app: &App) -> Option<String> {
    match app.input_destination {
        InputDestination::Buffer => None,
        InputDestination::Open => Some(format!(
            "Open file: {}",
            app.open_file_name.as_deref().unwrap_or("")
        )),
        InputDestination::Save => Some(format!(
            "Save as: {}",
            app.current_file_name.as_deref().unwrap_or("")
        )),
    }
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
