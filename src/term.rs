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

/// Represents an RGB color.
pub type Color = (i32, i32, i32);

/// Extension trait for RGB colors.
pub trait ColorExt {
    fn new() -> Self;
    fn is_valid(&self) -> bool;
}

impl ColorExt for Color {
    /// Creates an invalid/default color.
    ///
    /// Returns `(-1, -1, -1)` representing no color.
    fn new() -> Self {
        (-1, -1, -1)
    }

    /// Check if the color is valid
    ///
    /// Returns `true` if the color is valid (i.e., all components are between 0 and 255).
    fn is_valid(&self) -> bool {
        self.0 >= 0 && self.0 <= 255 && self.1 >= 0 && self.1 <= 255 && self.2 >= 0 && self.2 <= 255
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

    /// Show the cursor
    #[deprecated(since = "0.2.2", note = "Use `exec(cmd: Cmd)` method instead")]
    pub fn show_cursor() -> io::Result<()> {
        print!("{}", csi!("?25h"));
        io::stdout().flush()
    }

    /// Hide the cursor
    #[deprecated(since = "0.2.2", note = "Use `exec(cmd: Cmd)` method instead")]
    pub fn hide_cursor() -> io::Result<()> {
        print!("{}", csi!("?25l"));
        io::stdout().flush()
    }

    /// Move the cursor to the specified position
    #[deprecated(since = "0.2.2", note = "Use `exec(cmd: Cmd)` method instead")]
    pub fn move_cursor(x: usize, y: usize) -> io::Result<()> {
        print!("{}", csi!(&format!("{y};{x}H")));
        io::stdout().flush()
    }

    /// Clear the screen
    #[deprecated(since = "0.2.2", note = "Use `exec(cmd: Cmd)` method instead")]
    pub fn clear() -> io::Result<()> {
        print!("{}", csi!("2J"));
        io::stdout().flush()
    }

    /// Move the cursor to the home position
    #[deprecated(since = "0.2.2", note = "Use `exec(cmd: Cmd)` method instead")]
    pub fn move_cursor_to_home() -> io::Result<()> {
        print!("{}", csi!("H"));
        io::stdout().flush()
    }

    /// Enable alternative screen
    #[deprecated(since = "0.2.2", note = "Use `exec(cmd: Cmd)` method instead")]
    pub fn enable_alternative_screen() -> io::Result<()> {
        print!("{}", csi!("?1049h"));
        io::stdout().flush()
    }

    /// Disable alternative screen
    #[deprecated(since = "0.2.2", note = "Use `exec(cmd: Cmd)` method instead")]
    pub fn disable_alternative_screen() -> io::Result<()> {
        print!("{}", csi!("?1049l"));
        io::stdout().flush()
    }

    /// Enable mouse reporting
    #[deprecated(since = "0.2.2", note = "Use `exec(cmd: Cmd)` method instead")]
    pub fn enable_mouse_reporting() -> io::Result<()> {
        print!("{}", csi!("?1000h"));
        io::stdout().flush()
    }

    /// Disable mouse reporting
    #[deprecated(since = "0.2.2", note = "Use `exec(cmd: Cmd)` method instead")]
    pub fn disable_mouse_reporting() -> io::Result<()> {
        print!("{}", csi!("?1000l"));
        io::stdout().flush()
    }

    /// Enable SGR (Select Graphic Rendition) coordinates
    #[deprecated(since = "0.2.2", note = "Use `exec(cmd: Cmd)` method instead")]
    pub fn enable_sgr_coords() -> io::Result<()> {
        print!("{}", csi!("?1006h"));
        io::stdout().flush()
    }

    /// Disable SGR (Select Graphic Rendition) coordinates
    #[deprecated(since = "0.2.2", note = "Use `exec(cmd: Cmd)` method instead")]
    pub fn disable_sgr_coords() -> io::Result<()> {
        print!("{}", csi!("?1006l"));
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
        let color = Color::new();
        assert_eq!(color, (-1, -1, -1));
    }

    #[test]
    fn test_color_is_valid() {
        let color = Color::new();
        assert!(!color.is_valid());

        let color = (255, 255, 255);
        assert!(color.is_valid());

        let color = (-1, 128, 128);
        assert!(!color.is_valid());

        let color = (256, 128, 128);
        assert!(!color.is_valid());
    }
}
