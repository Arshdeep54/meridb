use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;

pub struct History {
    commands: VecDeque<String>,
    max_size: usize,
    current_index: Option<usize>,
    history_file: Option<PathBuf>,
}

impl History {
    pub fn new(max_size: usize) -> Self {
        History {
            commands: VecDeque::with_capacity(max_size),
            max_size,
            current_index: None,
            history_file: None,
        }
    }

    pub fn with_file(max_size: usize, history_file: PathBuf) -> Self {
        let mut history = History {
            commands: VecDeque::with_capacity(max_size),
            max_size,
            current_index: None,
            history_file: Some(history_file),
        };
        history.load_from_file().ok();
        history
    }

    pub fn add(&mut self, command: String) {
        if command.trim().is_empty() {
            return;
        }

        if self.commands.back() != Some(&command) {
            if self.commands.len() >= self.max_size {
                self.commands.pop_front();
            }
            self.commands.push_back(command);
            self.save_to_file().ok();
        }
        self.reset_position();
    }

    pub fn previous(&mut self) -> Option<&String> {
        if self.commands.is_empty() {
            return None;
        }

        match self.current_index {
            None => {
                self.current_index = Some(self.commands.len() - 1);
                self.commands.back()
            }
            Some(index) if index > 0 => {
                self.current_index = Some(index - 1);
                self.commands.get(index - 1)
            }
            _ => self.commands.front(),
        }
    }

    pub fn next_command(&mut self) -> Option<&String> {
        match self.current_index {
            Some(index) if index < self.commands.len() - 1 => {
                self.current_index = Some(index + 1);
                self.commands.get(index + 1)
            }
            Some(_) => {
                self.reset_position();
                None
            }
            None => None,
        }
    }

    pub fn reset_position(&mut self) {
        self.current_index = None;
    }

    fn load_from_file(&mut self) -> io::Result<()> {
        let Some(path) = &self.history_file else {
            return Ok(());
        };

        let file = match File::open(path) {
            Ok(file) => file,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(e),
        };

        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            if !line.trim().is_empty() {
                if self.commands.len() >= self.max_size {
                    self.commands.pop_front();
                }
                self.commands.push_back(line);
            }
        }
        Ok(())
    }

    fn save_to_file(&self) -> io::Result<()> {
        let Some(path) = &self.history_file else {
            return Ok(());
        };

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        for cmd in &self.commands {
            writeln!(file, "{}", cmd)?;
        }
        Ok(())
    }
}
