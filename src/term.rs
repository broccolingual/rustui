use bitflags::bitflags;
use nix::libc;
use nix::sys::termios::{self, LocalFlags, SetArg, Termios};
use std::os::unix::io::BorrowedFd;
use std::{
    io::{self, Write},
    os::fd::AsRawFd,
};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Attr: u16 {
        const NORMAL = 1;
        const BOLD = 2;
        const THIN = 4;
        const ITALIC = 8;
        const UNDERLINE = 16;
        const BLINK = 32;
        const FASTBLINK = 64;
        const INVERT = 128;
        const HIDDEN = 256;
        const REMOVE = 512;
    }
}

impl Attr {
    /// Convert attributes to ANSI escape codes
    pub fn to_ansi(&self) -> String {
        let mut ansi = String::from("\x1B[");
        if self.is_empty() {
            ansi.push_str("0m");
            return ansi;
        }
        if self.contains(Attr::NORMAL) {
            ansi.push_str("0;");
        }
        if self.contains(Attr::BOLD) {
            ansi.push_str("1;");
        }
        if self.contains(Attr::THIN) {
            ansi.push_str("2;");
        }
        if self.contains(Attr::ITALIC) {
            ansi.push_str("3;");
        }
        if self.contains(Attr::UNDERLINE) {
            ansi.push_str("4;");
        }
        if self.contains(Attr::BLINK) {
            ansi.push_str("5;");
        }
        if self.contains(Attr::FASTBLINK) {
            ansi.push_str("6;");
        }
        if self.contains(Attr::INVERT) {
            ansi.push_str("7;");
        }
        if self.contains(Attr::HIDDEN) {
            ansi.push_str("8;");
        }
        if self.contains(Attr::REMOVE) {
            ansi.push_str("9;");
        }
        ansi.pop(); // Remove the last semicolon
        ansi.push_str("m");
        ansi
    }
}

pub type Color = (i32, i32, i32);

#[doc(hidden)]
pub trait ColorExt {
    fn new() -> Self;
    fn is_valid(&self) -> bool;
}

impl ColorExt for Color {
    fn new() -> Self {
        (-1, -1, -1)
    }

    /// Check if the color is valid
    fn is_valid(&self) -> bool {
        self.0 >= 0 && self.0 <= 255 && self.1 >= 0 && self.1 <= 255 && self.2 >= 0 && self.2 <= 255
    }
}

pub struct Terminal {
    fd: i32,
    original: Termios,
}

impl Terminal {
    /// Enable raw mode
    pub fn enable_raw_mode(fd: i32) -> nix::Result<Self> {
        let borrowed_fd = unsafe { BorrowedFd::borrow_raw(fd) };
        let original = termios::tcgetattr(borrowed_fd)?;
        let mut raw = original.clone();

        raw.local_flags
            .remove(LocalFlags::ICANON | LocalFlags::ECHO);
        termios::tcsetattr(borrowed_fd, SetArg::TCSANOW, &raw)?;

        Ok(Terminal { fd, original })
    }

    /// Set the terminal to non-blocking mode
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

    /// Show the cursor
    pub fn show_cursor() -> io::Result<()> {
        print!("\x1B[?25h");
        io::stdout().flush()
    }

    /// Hide the cursor
    pub fn hide_cursor() -> io::Result<()> {
        print!("\x1B[?25l");
        io::stdout().flush()
    }

    /// Clear the screen
    pub fn clear() -> io::Result<()> {
        print!("\x1B[2J");
        io::stdout().flush()
    }

    /// Move the cursor to the home position
    pub fn move_to_home() -> io::Result<()> {
        print!("\x1B[H");
        io::stdout().flush()
    }

    /// Enable alternative screen
    pub fn enable_alternative_screen() -> io::Result<()> {
        print!("\x1B[?1049h");
        io::stdout().flush()
    }

    /// Disable alternative screen
    pub fn disable_alternative_screen() -> io::Result<()> {
        print!("\x1B[?1049l");
        io::stdout().flush()
    }

    /// Get the terminal size
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
