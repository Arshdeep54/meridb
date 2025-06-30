use libc::STDIN_FILENO;
use std::io::{self, Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW};

pub struct RawTerminal {
    original_termios: Termios,
}

impl RawTerminal {
    pub fn new() -> io::Result<Self> {
        let original_termios = Termios::from_fd(STDIN_FILENO)?;
        Ok(RawTerminal { original_termios })
    }

    pub fn raw_mode(&self) -> io::Result<()> {
        let mut raw = self.original_termios;
        raw.c_lflag &= !(ICANON | ECHO);
        tcsetattr(STDIN_FILENO, TCSANOW, &raw)?;
        Ok(())
    }
}

impl Drop for RawTerminal {
    fn drop(&mut self) {
        tcsetattr(STDIN_FILENO, TCSANOW, &self.original_termios).ok();
    }
}

#[derive(Debug, PartialEq)]
pub enum KeyEvent {
    Char(char),
    CtrlC,
    CtrlD,
    Backspace,
    Enter,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Unknown,
}

pub struct TerminalReader {
    stdin: io::Stdin,
    running: Arc<AtomicBool>,
}

impl TerminalReader {
    pub fn new() -> Self {
        TerminalReader {
            stdin: io::stdin(),
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn read_key(&mut self) -> io::Result<KeyEvent> {
        let mut buffer = [0; 4];
        match self.stdin.read(&mut buffer[..1])? {
            0 => Ok(KeyEvent::CtrlD),
            1 => match buffer[0] {
                3 => Ok(KeyEvent::CtrlC),
                13 | 10 => Ok(KeyEvent::Enter),
                127 => Ok(KeyEvent::Backspace),
                27 => {
                    if self.stdin.read(&mut buffer[1..3])? == 2 && buffer[1] == b'[' {
                        match buffer[2] {
                            b'A' => Ok(KeyEvent::ArrowUp),
                            b'B' => Ok(KeyEvent::ArrowDown),
                            b'C' => Ok(KeyEvent::ArrowRight),
                            b'D' => Ok(KeyEvent::ArrowLeft),
                            _ => Ok(KeyEvent::Unknown),
                        }
                    } else {
                        Ok(KeyEvent::Unknown)
                    }
                }
                c if c.is_ascii() => Ok(KeyEvent::Char(c as char)),
                _ => Ok(KeyEvent::Unknown),
            },
            _ => Ok(KeyEvent::Unknown),
        }
    }

    #[allow(dead_code)]
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

pub fn clear_line() {
    print!("\r\x1b[K");
    io::stdout().flush().ok();
}

#[allow(dead_code)]
pub fn move_cursor_to_start() {
    print!("\r");
    io::stdout().flush().ok();
}
