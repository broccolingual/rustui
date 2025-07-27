use std::io;
use std::os::unix::io::AsRawFd;
use nix::sys::termios::{Termios};
use std::sync::{Arc, Mutex, MutexGuard, TryLockError};
use std::{thread, time};

use crate::term;
use crate::framebuffer;

const RENDERING_RATE: u64 = 10; // ms

pub struct Window {
  pub width: usize,
  pub height: usize,
  pub front_fb: Arc<Mutex<framebuffer::Framebuffer>>,
  pub back_fb: Arc<Mutex<framebuffer::Framebuffer>>,
  pub fd: i32,
  pub orig_term: Option<Termios>,
}

impl Window {
  pub fn new() -> Self {
    let (width, height) = term::get_terminal_size().expect("Could not get terminal size");
    let front_fb = Arc::new(Mutex::new(framebuffer::Framebuffer::new(width, height)));
    let back_fb = Arc::new(Mutex::new(framebuffer::Framebuffer::new(width, height)));
    Self { width, height, front_fb, back_fb, fd: 0, orig_term: None }
  }

  pub fn init(&mut self) {
    let fd = io::stdin().as_raw_fd();
    self.fd = fd;
    self.orig_term = Some(term::enable_raw_mode(fd));
    term::enable_alternative_screen();
    term::hide_cursor();
  }

  pub fn start(&mut self) {
    let front_fb = Arc::clone(&self.front_fb);
    let back_fb = Arc::clone(&self.back_fb);
    
    thread::spawn(move || {
      loop {
        match back_fb.try_lock() {
          Ok(back_locked) => {
            if let Ok(mut front_locked) = front_fb.lock() {
              front_locked.refresh(&back_locked);
            }
          }
          Err(TryLockError::WouldBlock) => {}
          Err(_) => ()
        }
        thread::sleep(time::Duration::from_millis(RENDERING_RATE));
      }
    });
  }

  pub fn get_mutex_lock(&mut self) -> MutexGuard<'_, framebuffer::Framebuffer> {
    self.back_fb.lock().unwrap()
  }

  pub fn end(&mut self) {
    term::show_cursor();
    term::disable_alternative_screen();
    if let Some(orig) = &self.orig_term {
      term::disable_raw_mode(self.fd, orig);
    }
  }
}