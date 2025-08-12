use bitflags::bitflags;
use nix::libc;
use nix::sys::termios::{self, LocalFlags, SetArg, Termios};
use std::os::unix::io::BorrowedFd;
use std::{
    io::{self, Write},
    os::fd::AsRawFd,
};

/// Create a CSI (Control Sequence Introducer) escape sequence
#[macro_export]
macro_rules! csi {
    ($x:expr) => {
        String::from("\x1B[") + $x
    };
}

bitflags! {
    /// Represents terminal attributes using bitflags.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Attr: u16 {
        const NORMAL = 1; // 0
        const BOLD = 2; // 1
        const THIN = 4; // 2
        const ITALIC = 8; // 3
        const UNDERLINE = 16; // 4
        const BLINK = 32; // 5
        const FASTBLINK = 64; // 6
        const INVERT = 128; // 7
        const HIDDEN = 256; // 8
        const REMOVE = 512; // 9
        const PRIMARY = 1024; // 10
    }
}

impl Default for Attr {
    fn default() -> Self {
        Attr::NORMAL
    }
}

impl Attr {
    /// Convert attributes to ANSI escape codes
    ///
    /// Returns a string containing the ANSI escape codes for the active attributes.
    pub fn to_ansi(&self) -> String {
        if self.is_empty() {
            return csi!("0m");
        }

        let ansi_codes = [
            (Attr::NORMAL, "0"),
            (Attr::BOLD, "1"),
            (Attr::THIN, "2"),
            (Attr::ITALIC, "3"),
            (Attr::UNDERLINE, "4"),
            (Attr::BLINK, "5"),
            (Attr::FASTBLINK, "6"),
            (Attr::INVERT, "7"),
            (Attr::HIDDEN, "8"),
            (Attr::REMOVE, "9"),
            (Attr::PRIMARY, "10"),
        ]
        .iter()
        .filter(|(attr, _)| self.contains(*attr))
        .map(|(_, code)| *code)
        .collect::<Vec<_>>()
        .join(";");

        csi!(&format!("{ansi_codes}m"))
    }
}

/// Represents a color in the terminal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    RGB(u8, u8, u8),
    HSV(u8, u8, u8),
    None,
}

impl Default for Color {
    fn default() -> Self {
        Color::None
    }
}

impl Color {
    /// Convert the color to an ANSI escape code.
    ///
    /// Returns an ANSI escape code string for the color.
    pub fn to_ansi(&self, fg: bool) -> String {
        match self {
            Color::Black => csi!(if fg { "30m" } else { "40m" }),
            Color::Red => csi!(if fg { "31m" } else { "41m" }),
            Color::Green => csi!(if fg { "32m" } else { "42m" }),
            Color::Yellow => csi!(if fg { "33m" } else { "43m" }),
            Color::Blue => csi!(if fg { "34m" } else { "44m" }),
            Color::Magenta => csi!(if fg { "35m" } else { "45m" }),
            Color::Cyan => csi!(if fg { "36m" } else { "46m" }),
            Color::White => csi!(if fg { "37m" } else { "47m" }),
            Color::RGB(r, g, b) => {
                if fg {
                    let s = format!("38;2;{};{};{}m", r, g, b);
                    csi!(&s)
                } else {
                    let s = format!("48;2;{};{};{}m", r, g, b);
                    csi!(&s)
                }
            }
            Color::HSV(h, s, v) => {
                let norm_h = (*h as f32 / 255.0) * 360.0;
                let norm_s = *s as f32 / 255.0;
                let norm_v = *v as f32 / 255.0;
                let c = norm_v * norm_s;
                let x = c * (1.0 - ((norm_h / 60.0) % 2.0 - 1.0).abs());
                let m = norm_v - c;
                let mut r;
                let mut g;
                let mut b;
                if norm_h < 60.0 {
                    r = c;
                    g = x;
                    b = 0.0;
                } else if norm_h < 120.0 {
                    r = x;
                    g = c;
                    b = 0.0;
                } else if norm_h < 180.0 {
                    r = 0.0;
                    g = c;
                    b = x;
                } else if norm_h < 240.0 {
                    r = 0.0;
                    g = x;
                    b = c;
                } else if norm_h < 300.0 {
                    r = x;
                    g = 0.0;
                    b = c;
                } else {
                    r = c;
                    g = 0.0;
                    b = x;
                }
                r = (r + m) * 255.0;
                g = (g + m) * 255.0;
                b = (b + m) * 255.0;
                if fg {
                    let s = format!("38;2;{};{};{}m", r as u8, g as u8, b as u8);
                    csi!(&s)
                } else {
                    let s = format!("48;2;{};{};{}m", r as u8, g as u8, b as u8);
                    csi!(&s)
                }
            }
            Color::None => csi!(if fg { "39m" } else { "49m" }),
        }
    }
}

/// Represents terminal commands.
pub enum Cmd {
    ShowCursor,
    HideCursor,
    MoveCursor(usize, usize),
    MoveCursorToHome,
    ClearScreen,
    EnableAlternativeScreen,
    DisableAlternativeScreen,
    EnableMouseReporting,
    DisableMouseReporting,
    EnableSgrCoords,
    DisableSgrCoords,
}

/// Represents a terminal.
pub struct Terminal {
    /// The file descriptor for the terminal.
    fd: i32,
    /// The original terminal settings.
    original: Termios,
}

impl Terminal {
    /// Enable raw mode
    ///
    /// * `fd` - The file descriptor for the terminal.
    ///
    /// Returns a `Terminal` instance with raw mode enabled.
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
    ///
    /// Returns `Ok(())` if successful, or an error if it fails.
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

    /// Execute a terminal command
    ///
    /// * `cmd` - The command to execute.
    ///
    /// Returns `Ok(())` if successful, or an error if it fails.
    pub fn exec(cmd: Cmd) -> io::Result<()> {
        let ansi = match cmd {
            Cmd::ShowCursor => csi!("?25h"),
            Cmd::HideCursor => csi!("?25l"),
            Cmd::MoveCursor(x, y) => csi!(&format!("{y};{x}H")),
            Cmd::MoveCursorToHome => csi!("H"),
            Cmd::ClearScreen => csi!("2J"),
            Cmd::EnableAlternativeScreen => csi!("?1049h"),
            Cmd::DisableAlternativeScreen => csi!("?1049l"),
            Cmd::EnableMouseReporting => csi!("?1000h"),
            Cmd::DisableMouseReporting => csi!("?1000l"),
            Cmd::EnableSgrCoords => csi!("?1006h"),
            Cmd::DisableSgrCoords => csi!("?1006l"),
        };
        print!("{ansi}");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attr_to_ansi() {
        let attr = Attr::BOLD | Attr::UNDERLINE;
        assert_eq!(attr.to_ansi(), "\x1B[1;4m");

        let attr = Attr::empty();
        assert_eq!(attr.to_ansi(), "\x1B[0m");

        let attr = Attr::all();
        assert_eq!(attr.to_ansi(), "\x1B[0;1;2;3;4;5;6;7;8;9;10m");
    }

    #[test]
    fn test_csi_macro() {
        assert_eq!(csi!("?25h"), "\x1B[?25h");
    }

    #[test]
    fn test_color_init() {
        let color = Color::default();
        assert_eq!(color, Color::None);
    }

    #[test]
    fn test_color_is_valid() {
        assert!(Color::Black.to_ansi(true).contains("30m"));
        assert!(Color::Red.to_ansi(false).contains("41m"));
        assert!(Color::RGB(255, 0, 0).to_ansi(true).contains("38;2;255;0;0"));
        assert!(Color::HSV(0, 255, 255)
            .to_ansi(true)
            .contains("38;2;255;0;0"));
    }
}
