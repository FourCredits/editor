use std::{fmt::Display, fs, io};

#[derive(Debug)]
pub struct State {
    pub current_file_name: Option<String>,
    pub open_file_name: Option<String>,
    pub input_destination: InputDestination,
    pub file_contents: String,
    messages: Vec<String>,
    message_visible: bool,
    pub exited: bool,
    pub cursor: usize,
}

impl Default for State {
    fn default() -> Self {
        State {
            current_file_name: None,
            open_file_name: None,
            input_destination: InputDestination::Buffer,
            file_contents: String::new(),
            messages: Vec::new(),
            message_visible: false,
            exited: false,
            cursor: 0,
        }
    }
}

// input
impl State {
    pub fn accept_input(&mut self, input: Input) {
        match input {
            Input::None => (),
            Input::Backspace => self.backspace(),
            Input::Enter => {
                if let Err(err) = self.enter() {
                    self.add_message(err.to_string());
                }
            }
            Input::Cancel => {
                self.exited = true;
            }
            Input::NormalChar(c) => self.add_char(c),
            Input::Save => self.input_destination = InputDestination::Save,
            Input::Open => self.input_destination = InputDestination::Open,
            Input::New => self.new_file(),
            Input::ClearMessage => self.clear_message(),
            Input::MoveLeft => self.move_cursor_left(),
            Input::MoveRight => self.move_cursor_right(),
        }
    }

    fn backspace(&mut self) {
        match self.input_destination {
            InputDestination::Buffer => {
                if self.cursor > 0 {
                    self.move_cursor_left();
                    _ = self.file_contents.remove(self.cursor);
                }
            }
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
                self.add_char('\n');
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

    fn add_char(&mut self, c: char) {
        match self.input_destination {
            InputDestination::Buffer => {
                self.file_contents.insert(self.cursor, c);
                self.move_cursor_right();
            }
            InputDestination::Open => self.open_file_name.get_or_insert_with(String::new).push(c),
            InputDestination::Save => self
                .current_file_name
                .get_or_insert_with(String::new)
                .push(c),
        }
    }
}

// cursor
impl State {
    fn move_cursor_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    fn move_cursor_right(&mut self) {
        self.cursor = self
            .cursor
            .saturating_add(1)
            .min(self.file_contents.chars().count());
    }
}

// files
impl State {
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
    fn new_file(&mut self) {
        self.file_contents.clear();
        self.current_file_name = None;
    }
}

// messages
impl State {
    fn add_message(&mut self, message: String) {
        self.messages.push(message);
        self.message_visible = true;
    }

    fn clear_message(&mut self) {
        self.message_visible = false;
    }

    pub fn latest_message(&self) -> Option<&str> {
        self.messages
            .last()
            .filter(|_| self.message_visible)
            .map(String::as_str)
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum InputDestination {
    #[default]
    Buffer,
    Open,
    Save,
}

pub enum Input {
    None,
    Backspace,
    Enter,
    Cancel,
    NormalChar(char),
    Save,
    Open,
    New,
    ClearMessage,
    MoveLeft,
    MoveRight,
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
