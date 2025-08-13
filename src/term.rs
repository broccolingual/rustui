use bitflags::bitflags;
use nix::libc;
use nix::sys::termios::{self, LocalFlags, SetArg, Termios};
use std::os::unix::io::{BorrowedFd, RawFd};
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

        let attr_mappings = [
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
        ];

        let mut buf = String::with_capacity(24);
        buf.push_str("\x1B[");

        let mut first = true;
        for (flag, code) in attr_mappings.iter() {
            if self.contains(*flag) {
                if !first {
                    buf.push(';');
                }
                buf.push_str(code);
                first = false;
            }
        }

        buf.push('m');
        buf
    }
}

/// Represents a color in the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
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
    #[default]
    None,
}

impl Color {
    /// Convert the color to an ANSI escape code.
    ///
    /// Returns an ANSI escape code string for the color.
    pub fn to_ansi(&self, fg: bool) -> String {
        use std::fmt::Write;
        let mut buf = String::with_capacity(20);
        buf.push_str("\x1B[");

        match self {
            Color::Black => buf.push_str(if fg { "30m" } else { "40m" }),
            Color::Red => buf.push_str(if fg { "31m" } else { "41m" }),
            Color::Green => buf.push_str(if fg { "32m" } else { "42m" }),
            Color::Yellow => buf.push_str(if fg { "33m" } else { "43m" }),
            Color::Blue => buf.push_str(if fg { "34m" } else { "44m" }),
            Color::Magenta => buf.push_str(if fg { "35m" } else { "45m" }),
            Color::Cyan => buf.push_str(if fg { "36m" } else { "46m" }),
            Color::White => buf.push_str(if fg { "37m" } else { "47m" }),
            Color::RGB(r, g, b) => {
                let _ = write!(buf, "{};2;{};{};{}m", if fg { "38" } else { "48" }, r, g, b);
            }
            Color::HSV(h, s, v) => {
                let h = *h as u32 * 360 / 255;
                let s = *s as u32;
                let v = *v as u32;
                let c = v * s / 255;
                let h_mod = (h % 120) as i32 - 60;
                let x = c * (60 - h_mod.unsigned_abs()) / 60;
                let m = v - c;
                let (r, g, b) = match h {
                    0..=59 => (c, x, 0),
                    60..=119 => (x, c, 0),
                    120..=179 => (0, c, x),
                    180..=239 => (0, x, c),
                    240..=299 => (x, 0, c),
                    _ => (c, 0, x),
                };
                let r = (r + m).min(255);
                let g = (g + m).min(255);
                let b = (b + m).min(255);
                let _ = write!(buf, "{};2;{};{};{}m", if fg { "38" } else { "48" }, r, g, b);
            }
            Color::None => buf.push_str(if fg { "39m" } else { "49m" }),
        }
        buf
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
    fd: RawFd,
    /// The original terminal settings.
    original: Option<Termios>,
}

impl Terminal {
    /// Create a new terminal instance.
    ///
    /// Returns a new `Terminal` instance.
    pub fn new() -> Self {
        let fd: RawFd = std::io::stdout().as_raw_fd();
        Terminal { fd, original: None }
    }

    /// Get a borrowed file descriptor with error handling.
    ///
    /// Returns a `BorrowedFd` for the terminal file descriptor.
    fn get_borrowed_fd(&self) -> nix::Result<BorrowedFd<'_>> {
        if self.fd < 0 {
            return Err(nix::Error::EBADF);
        }
        Ok(unsafe { BorrowedFd::borrow_raw(self.fd) })
    }

    /// Enable raw mode
    ///
    /// Returns a `Terminal` instance with raw mode enabled.
    pub fn enable_raw_mode(&mut self) -> nix::Result<()> {
        let borrowed_fd = self.get_borrowed_fd()?;
        let original = termios::tcgetattr(borrowed_fd)?;
        let mut raw = original.clone();

        raw.local_flags
            .remove(LocalFlags::ICANON | LocalFlags::ECHO);
        termios::tcsetattr(borrowed_fd, SetArg::TCSANOW, &raw)?;

        self.original = Some(original);
        Ok(())
    }

    /// Disable raw mode
    ///
    /// Returns `Ok(())` if successful, or an error if it fails.
    pub fn disable_raw_mode(&mut self) -> nix::Result<()> {
        if let Some(original) = &self.original {
            let borrowed_fd = self.get_borrowed_fd()?;
            termios::tcsetattr(borrowed_fd, SetArg::TCSANOW, original)?;
            self.original = None;
        }
        Ok(())
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
    pub fn get_size(&self) -> io::Result<(usize, usize)> {
        let mut ws = libc::winsize {
            ws_row: 0,
            ws_col: 0,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };

        unsafe {
            if libc::ioctl(self.fd, libc::TIOCGWINSZ, &mut ws) == 0 {
                Ok((ws.ws_col as usize, ws.ws_row as usize))
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.disable_raw_mode();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csi_macro() {
        assert_eq!(csi!("?25h"), "\x1B[?25h");
    }

    #[test]
    fn test_attr_default() {
        let attr = Attr::default();
        assert_eq!(attr, Attr::NORMAL);
    }

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
    fn test_color_default() {
        let color = Color::default();
        assert_eq!(color, Color::None);
    }

    #[test]
    fn test_color_to_ansi() {
        assert!(Color::Black.to_ansi(true).contains("30m"));
        assert!(Color::Red.to_ansi(false).contains("41m"));
        assert!(Color::RGB(255, 0, 0).to_ansi(true).contains("38;2;255;0;0"));
        assert!(Color::HSV(0, 255, 255)
            .to_ansi(true)
            .contains("38;2;255;0;0"));
    }
}
