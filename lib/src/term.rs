use bitflags::bitflags;
use nix::libc;
use nix::sys::termios::{self, LocalFlags, SetArg, Termios};
use std::ops::BitOr;
use std::os::unix::io::BorrowedFd;
use std::{
    io::{self, Write},
    os::fd::AsRawFd,
};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Attr: u16 {
        const NORMAL    = 0b0000000000000001;
        const BOLD      = 0b0000000000000010;
        const THIN      = 0b0000000000000100;
        const ITALIC    = 0b0000000000001000;
        const UNDERLINE = 0b0000000000010000;
        const BLINK     = 0b0000000000100000;
        const FASTBLINK = 0b0000000001000000;
        const INVERT    = 0b0000000010000000;
        const HIDDEN    = 0b0000000100000000;
        const REMOVE    = 0b0000001000000000;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    Fg(u8),
    Bg(u8),
    Fg24(u8),
    Bg24(u8),
    FgRgb(u8, u8, u8),
    BgRgb(u8, u8, u8),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub attr: Attr,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
}

impl Style {
    /// コンストラクタ
    pub fn new() -> Self {
        Self {
            attr: Attr::NORMAL,
            fg: None,
            bg: None,
        }
    }

    pub fn to_ansi(&self) -> String {
        let mut code: Vec<String> = Vec::new();
        if self.attr.contains(Attr::NORMAL) {
            code.push("0".to_string());
        }
        if self.attr.contains(Attr::BOLD) {
            code.push("1".to_string());
        }
        if self.attr.contains(Attr::THIN) {
            code.push("2".to_string());
        }
        if self.attr.contains(Attr::ITALIC) {
            code.push("3".to_string());
        }
        if self.attr.contains(Attr::UNDERLINE) {
            code.push("4".to_string());
        }
        if self.attr.contains(Attr::BLINK) {
            code.push("5".to_string());
        }
        if self.attr.contains(Attr::FASTBLINK) {
            code.push("6".to_string());
        }
        if self.attr.contains(Attr::INVERT) {
            code.push("7".to_string());
        }
        if self.attr.contains(Attr::HIDDEN) {
            code.push("8".to_string());
        }
        if self.attr.contains(Attr::REMOVE) {
            code.push("9".to_string());
        }
        if let Some(Color::Fg(c)) = self.fg {
            if c < 8 {
                code.push(format!("{}", c + 30));
            } else {
                code.push(format!("38;5;{}", c));
            }
        }
        if let Some(Color::Bg(c)) = self.bg {
            if c < 8 {
                code.push(format!("{}", c + 40));
            }
        }
        if let Some(Color::Fg24(c)) = self.fg {
            code.push(format!("38;5;{}", c));
        }
        if let Some(Color::Bg24(c)) = self.bg {
            code.push(format!("48;5;{}", c));
        }
        if let Some(Color::FgRgb(r, g, b)) = self.fg {
            code.push(format!("38;2;{};{};{}", r, g, b).to_string());
        }
        if let Some(Color::BgRgb(r, g, b)) = self.bg {
            code.push(format!("48;2;{};{};{}", r, g, b).to_string());
        }
        if code.is_empty() {
            "\x1B[0m".to_string() // Reset
        } else {
            format!("\x1B[{}m", code.join(";"))
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Self::new()
    }
}

impl BitOr<Attr> for Style {
    type Output = Self;

    fn bitor(mut self, rhs: Attr) -> Self::Output {
        self.attr |= rhs;
        self
    }
}

impl BitOr<Color> for Style {
    type Output = Self;

    fn bitor(mut self, rhs: Color) -> Self::Output {
        match rhs {
            Color::Fg(c) => self.fg = Some(Color::Fg(c)),
            Color::Bg(c) => self.bg = Some(Color::Bg(c)),
            Color::Fg24(c) => self.fg = Some(Color::Fg24(c)),
            Color::Bg24(c) => self.bg = Some(Color::Bg24(c)),
            Color::FgRgb(r, g, b) => self.fg = Some(Color::FgRgb(r, g, b)),
            Color::BgRgb(r, g, b) => self.bg = Some(Color::BgRgb(r, g, b)),
        }
        self
    }
}

#[macro_export]
macro_rules! style {
  ($($x:expr),*) => {
    {
      let mut s= $crate::term::Style::default();
      $(
        s = s | $x;
      )*
      s
    }
  };
}

pub struct Terminal {
    fd: i32,
    original: Termios,
}

impl Terminal {
    pub fn enable(fd: i32) -> nix::Result<Self> {
        let borrowed_fd = unsafe { BorrowedFd::borrow_raw(fd) };
        let original = termios::tcgetattr(borrowed_fd)?;
        let mut raw = original.clone();

        raw.local_flags
            .remove(LocalFlags::ICANON | LocalFlags::ECHO);
        termios::tcsetattr(borrowed_fd, SetArg::TCSANOW, &raw)?;

        Ok(Terminal { fd, original })
    }

    pub fn set_nonblocking(&self) -> nix::Result<()> {
        unsafe {
            let flags = libc::fcntl(self.fd, libc::F_GETFL);
            if flags == -1 {
                return Err(nix::Error::last());
            }
            let result = libc::fcntl(self.fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
            if result == -1 {
                return Err(nix::Error::last());
            }
        }
        Ok(())
    }

    // 静的なターミナル制御メソッド
    pub fn show_cursor() -> io::Result<()> {
        print!("\x1B[?25h");
        io::stdout().flush()
    }

    pub fn hide_cursor() -> io::Result<()> {
        print!("\x1B[?25l");
        io::stdout().flush()
    }

    pub fn clear() -> io::Result<()> {
        print!("\x1B[2J");
        io::stdout().flush()
    }

    pub fn move_to_home() -> io::Result<()> {
        print!("\x1B[H");
        io::stdout().flush()
    }

    pub fn enable_alternative_screen() -> io::Result<()> {
        print!("\x1B[?1049h");
        io::stdout().flush()
    }

    pub fn disable_alternative_screen() -> io::Result<()> {
        print!("\x1B[?1049l");
        io::stdout().flush()
    }

    pub fn get_size() -> io::Result<(usize, usize)> {
        let fd = std::io::stdout().as_raw_fd();
        let mut ws = libc::winsize {
            ws_row: 0,
            ws_col: 0,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };

        unsafe {
            if libc::ioctl(fd, libc::TIOCGWINSZ, &mut ws) == 0 {
                Ok((ws.ws_col as usize, ws.ws_row as usize))
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let borrowed_fd = unsafe { BorrowedFd::borrow_raw(self.fd) };
        let _ = termios::tcsetattr(borrowed_fd, SetArg::TCSANOW, &self.original);
    }
}
