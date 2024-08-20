use std::io::{self, Write};
use crossterm::terminal::{self, ClearType};
use crossterm::{cursor, queue};

pub fn _clear_screen(stdout: &mut io::Stdout) -> io::Result<()> {
    queue!(
        stdout,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0)
    )?;
    stdout.flush()
}
