use crate::csi;
use nix::libc;
use nix::sys::termios::{self, LocalFlags, SetArg, Termios};
use std::os::unix::io::{BorrowedFd, RawFd};
use std::{
    io::{self, Write},
    os::fd::AsRawFd,
};

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
