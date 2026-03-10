use nix::libc;
use nix::sys::termios::{self, ControlFlags, InputFlags, LocalFlags, OutputFlags, SetArg, Termios};
use std::os::unix::io::{BorrowedFd, RawFd};
use std::{
    io::{self, Write},
    os::fd::AsRawFd,
};

/// Represents terminal commands.
pub(crate) enum Cmd {
    ShowCursor,
    HideCursor,
    ClearScreen,
    EnableAlternativeScreen,
    DisableAlternativeScreen,
    EnableMouseReporting,
    DisableMouseReporting,
    EnableSgrCoords,
    DisableSgrCoords,
}

/// Represents a terminal.
pub(crate) struct Terminal {
    /// The file descriptor for the terminal.
    fd: RawFd,
    /// The original terminal settings.
    original: Option<Termios>,
}

impl Terminal {
    /// Create a new terminal instance.
    ///
    /// Returns a new `Terminal` instance.
    pub(crate) fn new() -> Self {
        let fd: RawFd = std::io::stdout().as_raw_fd();
        Self { fd, original: None }
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
    pub(crate) fn enable_raw_mode(&mut self) -> nix::Result<()> {
        let borrowed_fd = self.get_borrowed_fd()?;
        let original = termios::tcgetattr(borrowed_fd)?;
        let mut raw = original.clone();

        raw.input_flags.remove(
            InputFlags::BRKINT
                | InputFlags::ICRNL
                | InputFlags::INPCK
                | InputFlags::ISTRIP
                | InputFlags::IXON,
        );
        raw.input_flags.insert(InputFlags::IUTF8); // correct multi-byte char handling on Backspace

        raw.output_flags.remove(OutputFlags::OPOST); // disable output processing

        raw.control_flags
            .remove(ControlFlags::CSIZE | ControlFlags::PARENB);
        raw.control_flags.insert(ControlFlags::CS8);

        raw.local_flags
            .remove(LocalFlags::ICANON | LocalFlags::ECHO | LocalFlags::ISIG | LocalFlags::IEXTEN);
        raw.control_chars[libc::VMIN] = 1; // Minimum number of characters to read
        raw.control_chars[libc::VTIME] = 0; // No timeout

        termios::tcsetattr(borrowed_fd, SetArg::TCSANOW, &raw)?;

        self.original = Some(original);
        Ok(())
    }

    /// Disable raw mode
    ///
    /// Returns `Ok(())` if successful, or an error if it fails.
    pub(crate) fn disable_raw_mode(&mut self) -> nix::Result<()> {
        if let Some(original) = &self.original {
            let borrowed_fd = self.get_borrowed_fd()?;
            termios::tcsetattr(borrowed_fd, SetArg::TCSANOW, original)?;
            self.original = None;
        }
        Ok(())
    }

    /// Set stdin to non-blocking mode.
    ///
    /// Note: targets `STDIN_FILENO`, not stdout. Setting O_NONBLOCK on stdout
    /// would risk EAGAIN errors during large write bursts (e.g. refresh).
    ///
    /// Returns `Ok(())` if successful, or an error if it fails.
    pub(crate) fn set_nonblocking(&self) -> nix::Result<()> {
        unsafe {
            let flags = libc::fcntl(libc::STDIN_FILENO, libc::F_GETFL);
            if flags == -1 {
                return Err(nix::Error::last());
            }
            let result = libc::fcntl(libc::STDIN_FILENO, libc::F_SETFL, flags | libc::O_NONBLOCK);
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
    pub(crate) fn exec(cmd: Cmd) -> io::Result<()> {
        let stdout = io::stdout();
        let mut lock = stdout.lock();
        match cmd {
            Cmd::ShowCursor => lock.write_all(b"\x1B[?25h")?,
            Cmd::HideCursor => lock.write_all(b"\x1B[?25l")?,
            Cmd::ClearScreen => lock.write_all(b"\x1B[2J")?,
            Cmd::EnableAlternativeScreen => lock.write_all(b"\x1B[?1049h")?,
            Cmd::DisableAlternativeScreen => lock.write_all(b"\x1B[?1049l")?,
            Cmd::EnableMouseReporting => lock.write_all(b"\x1B[?1000h")?,
            Cmd::DisableMouseReporting => lock.write_all(b"\x1B[?1000l")?,
            Cmd::EnableSgrCoords => lock.write_all(b"\x1B[?1006h")?,
            Cmd::DisableSgrCoords => lock.write_all(b"\x1B[?1006l")?,
        }
        lock.flush()
    }

    /// Get the terminal size
    pub(crate) fn get_size(&self) -> io::Result<(usize, usize)> {
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

impl Default for Terminal {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.disable_raw_mode();
    }
}
