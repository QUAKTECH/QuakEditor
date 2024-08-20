use std::io::{self, Write};
use std::fs;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{self, ClearType},
    cursor,
    style::{Color, SetForegroundColor, SetBackgroundColor},
    queue,
};

pub struct Editor {
    content: Vec<String>,
    filename: String,
    cursor: (usize, usize),
    saved: bool,
}

impl Editor {

    pub fn cleanup(&self) -> io::Result<()> {
        let mut stdout = io::stdout();
        queue!(
            stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )?;
        stdout.flush()?;
        terminal::disable_raw_mode()
    }



    pub fn new(filename: &str) -> Self {
        let content = fs::read_to_string(filename).unwrap_or_default();
        Editor {
            content: content.lines().map(String::from).collect(),
            filename: filename.to_string(),
            cursor: (0, 0),
            saved: true,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();

        self.refresh_display(&mut stdout)?;

        loop {
            if let Event::Key(key_event) = event::read()? {
                match (key_event.code, key_event.modifiers) {
                    (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                        self.save()?;
                        self.saved = true;
                    },
                    (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                        if !self.saved {
                            if self.confirm_exit(&mut stdout)? {
                                break;
                            }
                        } else {
                            break;
                        }
                    },
                    (KeyCode::Char('d'), KeyModifiers::CONTROL) => self.delete_current_line(),
                    (KeyCode::Char(c), _) => {
                        self.insert_char(c);
                        self.saved = false;
                    },
                    (KeyCode::Enter, _) => {
                        self.insert_newline();
                        self.saved = false;
                    },
                    (KeyCode::Backspace, _) => {
                        self.backspace();
                        self.saved = false;
                    },
                    (KeyCode::Left, _) => self.move_cursor_left(),
                    (KeyCode::Right, _) => self.move_cursor_right(),
                    (KeyCode::Up, _) => self.move_cursor_up(),
                    (KeyCode::Down, _) => self.move_cursor_down(),
                    _ => {}
                }
                self.refresh_display(&mut stdout)?;
            }
        }

        terminal::disable_raw_mode()?;
        Ok(())
    }

    fn confirm_exit(&self, stdout: &mut io::Stdout) -> io::Result<bool> {
        let (_cols, rows) = terminal::size()?;
        queue!(
            stdout,
            cursor::MoveTo(0, rows - 2),
            SetForegroundColor(Color::Black),
            SetBackgroundColor(Color::White),
            terminal::Clear(ClearType::CurrentLine)
        )?;
        let prompt = format!("Are you sure you want to exit {}? Changes will be lost. (y/n) > ", self.filename);
        write!(stdout, "{}", prompt)?;
        stdout.flush()?;

        loop {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => return Ok(true),
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        self.refresh_display(stdout)?;
                        return Ok(false);
                    },
                    _ => {}
                }
            }
        }
    }

   fn refresh_display(&self, stdout: &mut io::Stdout) -> io::Result<()> {
    let (cols, rows) = terminal::size()?;
    
    queue!(
        stdout,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0)
    )?;

    let line_number_width = self.content.len().to_string().len();

    for (idx, line) in self.content.iter().take(rows as usize - 1).enumerate() {
        queue!(
            stdout,
            cursor::MoveTo(0, idx as u16),
            SetForegroundColor(Color::DarkGrey),
            SetBackgroundColor(Color::Reset)
        )?;
        write!(stdout, "{:>width$} ", idx + 1, width = line_number_width)?;
        
        queue!(
            stdout,
            SetForegroundColor(Color::Reset)
        )?;
        write!(stdout, "{}", line)?;
    }

    // Status bar remains the same
    queue!(
        stdout,
        cursor::MoveTo(0, rows - 1),
        SetForegroundColor(Color::Black),
        SetBackgroundColor(Color::White),
        terminal::Clear(ClearType::CurrentLine)
    )?;
    let status = format!(
        "[ {} ]             Ctrl+S: Save | Ctrl+Q: Quit             QuakEditor Version 0.1.0",
        self.filename
    );
    let padding = " ".repeat(cols as usize - status.len());
    write!(stdout, "{}{}", status, padding)?;

    queue!(
        stdout,
        SetForegroundColor(Color::Reset),
        SetBackgroundColor(Color::Reset),
        cursor::MoveTo((self.cursor.1 + line_number_width + 1) as u16, self.cursor.0 as u16)
    )?;
    stdout.flush()
}
 

    fn save(&self) -> io::Result<()> {
        let content = self.content.join("\n");
        fs::write(&self.filename, content)?;
        let mut stdout = io::stdout();
        let (_, rows) = terminal::size()?;
        queue!(
            stdout,
            cursor::MoveTo(0, rows - 1),
            terminal::Clear(ClearType::CurrentLine),
            SetForegroundColor(Color::Green)
        )?;
        write!(stdout, "File saved successfully!")?;
        stdout.flush()?;
        std::thread::sleep(std::time::Duration::from_secs(1));
        Ok(())
    }

    fn insert_char(&mut self, c: char) {
        if self.content.is_empty() {
            self.content.push(String::new());
        }
        self.content[self.cursor.0].insert(self.cursor.1, c);
        self.cursor.1 += 1;
    }

    fn insert_newline(&mut self) {
        if self.cursor.0 == self.content.len() {
            self.content.push(String::new());
        } else {
            let current_line = &mut self.content[self.cursor.0];
            let new_line = current_line.split_off(self.cursor.1);
            self.content.insert(self.cursor.0 + 1, new_line);
        }
        self.cursor.0 += 1;
        self.cursor.1 = 0;
    }

    fn backspace(&mut self) {
        if self.cursor.1 > 0 {
            self.content[self.cursor.0].remove(self.cursor.1 - 1);
            self.cursor.1 -= 1;
        } else if self.cursor.0 > 0 {
            let current_line = self.content.remove(self.cursor.0);
            self.cursor.0 -= 1;
            self.cursor.1 = self.content[self.cursor.0].len();
            self.content[self.cursor.0].push_str(&current_line);
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor.1 > 0 {
            self.cursor.1 -= 1;
        } else if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
            self.cursor.1 = self.content[self.cursor.0].len();
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor.1 < self.content[self.cursor.0].len() {
            self.cursor.1 += 1;
        } else if self.cursor.0 < self.content.len() - 1 {
            self.cursor.0 += 1;
            self.cursor.1 = 0;
        }
    }

    fn move_cursor_up(&mut self) {
        if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
            self.cursor.1 = self.cursor.1.min(self.content[self.cursor.0].len());
        }
    }

    fn move_cursor_down(&mut self) {
        if self.cursor.0 < self.content.len() - 1 {
            self.cursor.0 += 1;
            self.cursor.1 = self.cursor.1.min(self.content[self.cursor.0].len());
        }
    }

    fn delete_current_line(&mut self) {
        if !self.content.is_empty() {
            self.content.remove(self.cursor.0);
            if self.content.is_empty() {
                self.content.push(String::new());
            }
            if self.cursor.0 >= self.content.len() {
                self.cursor.0 = self.content.len() - 1;
            }
            self.cursor.1 = 0;
        }
        self.saved = false;
    }
}
