use std::{io::{self, Write}, os::fd::AsRawFd};
use nix::libc::{self, fcntl, F_GETFL, F_SETFL, O_NONBLOCK};
use nix::sys::termios::{self, Termios, SetArg, LocalFlags};
use std::os::unix::io::BorrowedFd;

#[repr(C)]
#[derive(Debug)]
struct Winsize {
    ws_row: u16,
    ws_col: u16,
    ws_xpixel: u16,
    ws_ypixel: u16,
}

pub fn get_terminal_size() -> Option<(usize, usize)> {
  let fd = std::io::stdout().as_raw_fd();
  let mut ws = Winsize {
    ws_row: 0, ws_col: 0, ws_xpixel: 0, ws_ypixel: 0,
  };

  unsafe {
    if libc::ioctl(fd, libc::TIOCGWINSZ, &mut ws) == 0 {
      Some((ws.ws_col as usize, ws.ws_row as usize))
    } else {
      None
    }
  }
}

#[derive(Clone, PartialEq)]
pub enum Attr {
  Normal,
  Bold,
  Thin,
  Italic,
  Underline,
  Blink,
  FastBlink,
  Invert,
  Hidden,
  Remove,
  ForeColor(u8),
  BackColor(u8),
}

pub fn attr_to_ansi(attr: &Attr) -> String {
  match attr {
    Attr::Normal => "\x1B[0m".to_string(),
    Attr::Bold => "\x1B[1m".to_string(),
    Attr::Thin => "\x1B[2m".to_string(),
    Attr::Italic => "\x1B[3m".to_string(),
    Attr::Underline => "\x1B[4m".to_string(),
    Attr::Blink => "\x1B[5m".to_string(),
    Attr::FastBlink => "\x1B[6m".to_string(),
    Attr::Invert => "\x1B[7m".to_string(),
    Attr::Hidden => "\x1B[8m".to_string(),
    Attr::Remove => "\x1B[9m".to_string(),
    Attr::ForeColor(c) => format!("\x1B[38;5;{}m", c),
    Attr::BackColor(c) => format!("\x1B[48;5;{}m", c),
  }
}

pub fn show_cursor() {
  print!("\x1B[?25h");
  io::stdout().flush().unwrap();
}

pub fn hide_cursor() {
  print!("\x1B[?25l");
  io::stdout().flush().unwrap();
}

pub fn clear() {
  print!("\x1B[2J");
  io::stdout().flush().unwrap();
}

pub fn move_to_home() {
  print!("\x1B[H");
  io::stdout().flush().unwrap();
}

pub fn enable_alternative_screen() {
  print!("\x1B[?1049h");
  io::stdout().flush().unwrap();
}

pub fn disable_alternative_screen() {
  print!("\x1B[?1049l");
  io::stdout().flush().unwrap();
}

pub fn enable_raw_mode(fd: i32) -> Termios {
  let fd = unsafe { BorrowedFd::borrow_raw(fd) };
  let orig = termios::tcgetattr(fd).unwrap();
  let mut raw = orig.clone();
  raw.local_flags.remove(LocalFlags::ICANON | LocalFlags::ECHO);
  termios::tcsetattr(fd, SetArg::TCSANOW, &raw).unwrap();
  orig
}

pub fn disable_raw_mode(fd: i32, orig: &Termios) {
  let fd = unsafe { BorrowedFd::borrow_raw(fd) };
  termios::tcsetattr(fd, SetArg::TCSANOW, orig).unwrap();
}

pub fn set_nonblocking(fd: i32) {
  unsafe {
    let flags = fcntl(fd, F_GETFL);
    fcntl(fd, F_SETFL, flags | O_NONBLOCK);
  }
}
