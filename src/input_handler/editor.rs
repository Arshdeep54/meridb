use std::io::{self, Write};
use std::path::PathBuf;

use super::history::History;
use super::terminal::{clear_line, KeyEvent, RawTerminal, TerminalReader};

pub struct Editor {
    history: History,
    current_buffer: String,
    cursor_position: usize,
    terminal_reader: TerminalReader,
    _raw_terminal: RawTerminal,
}

pub struct InputHandler {
    editor: Editor,
}

impl Editor {
    pub fn new() -> io::Result<Self> {
        Ok(Editor {
            history: History::new(25),
            current_buffer: String::new(),
            cursor_position: 0,
            terminal_reader: TerminalReader::new(),
            _raw_terminal: RawTerminal::new()?,
        })
    }

    pub fn with_history_file(history_file: PathBuf) -> io::Result<Self> {
        Ok(Editor {
            history: History::with_file(25, history_file),
            current_buffer: String::new(),
            cursor_position: 0,
            terminal_reader: TerminalReader::new(),
            _raw_terminal: RawTerminal::new()?,
        })
    }

    pub fn readline(&mut self, prompt: &str) -> io::Result<String> {
        self._raw_terminal.raw_mode()?;
        self.current_buffer.clear();
        self.cursor_position = 0;

        print!("{}", prompt);
        io::stdout().flush()?;

        loop {
            match self.terminal_reader.read_key()? {
                KeyEvent::Char(c) => {
                    self.insert_char(c);
                    self.redraw_line(prompt)?;
                }
                KeyEvent::Backspace => {
                    self.handle_backspace();
                    self.redraw_line(prompt)?;
                }
                KeyEvent::ArrowLeft => {
                    self.move_cursor_left();
                    self.redraw_line(prompt)?;
                }
                KeyEvent::ArrowRight => {
                    self.move_cursor_right();
                    self.redraw_line(prompt)?;
                }
                KeyEvent::ArrowUp => {
                    if let Some(prev_cmd) = self.history.previous() {
                        self.current_buffer = prev_cmd.clone();
                        self.cursor_position = self.current_buffer.len();
                        self.redraw_line(prompt)?;
                    }
                }
                KeyEvent::ArrowDown => {
                    match self.history.next_command() {
                        Some(next_cmd) => {
                            self.current_buffer = next_cmd.clone();
                            self.cursor_position = self.current_buffer.len();
                        }
                        None => {
                            self.current_buffer.clear();
                            self.cursor_position = 0;
                        }
                    }
                    self.redraw_line(prompt)?;
                }
                KeyEvent::Enter => {
                    println!();
                    let command = self.current_buffer.clone();
                    self.history.add(command.clone());
                    return Ok(command);
                }
                KeyEvent::CtrlC => {
                    println!("^C");
                    self.current_buffer.clear();
                    self.cursor_position = 0;
                    return Ok(String::new());
                }
                KeyEvent::CtrlD => {
                    if self.current_buffer.is_empty() {
                        println!();
                        return Ok("exit".to_string());
                    }
                }
                _ => {}
            }
        }
    }

    fn insert_char(&mut self, c: char) {
        self.current_buffer.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    fn handle_backspace(&mut self) {
        if self.cursor_position > 0 {
            self.current_buffer.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor_position < self.current_buffer.len() {
            self.cursor_position += 1;
        }
    }

    fn redraw_line(&self, prompt: &str) -> io::Result<()> {
        clear_line();
        print!("{}{}", prompt, self.current_buffer);

        if self.cursor_position < self.current_buffer.len() {
            print!("\x1b[{}D", self.current_buffer.len() - self.cursor_position);
        }

        io::stdout().flush()
    }
}

impl InputHandler {
    pub fn new() -> io::Result<Self> {
        Ok(InputHandler {
            editor: Editor::new()?,
        })
    }

    pub fn with_history_file(history_file: PathBuf) -> io::Result<Self> {
        Ok(InputHandler {
            editor: Editor::with_history_file(history_file)?,
        })
    }

    pub fn readline(&mut self, prompt: &str) -> io::Result<String> {
        self.editor.readline(prompt)
    }
}
