use std::io::{self, Read};
use std::thread;
use std::time;
use std::sync::mpsc::{channel, Sender, Receiver};

const CAPTURING_RATE: u64 = 10; // ms

pub enum Key {
  ArrowUp,
  ArrowDown,
  ArrowRight,
  ArrowLeft,
  Char(char),
  Escape,
  Unknown,
}

pub fn read_key() -> Option<Key> {
  let mut stdin = io::stdin().lock();
  let mut buf = [0u8; 3];
  match stdin.read(&mut buf) {
    Ok(0) => None,
    Ok(n) => {
      match buf[0] {
        0x1B => {
          if n >= 3 && buf[1] == b'[' {
            match buf[2] {
              b'A' => Some(Key::ArrowUp),
              b'B' => Some(Key::ArrowDown),
              b'C' => Some(Key::ArrowRight),
              b'D' => Some(Key::ArrowLeft),
              _ => Some(Key::Unknown),
            }
          } else {
            Some(Key::Escape)
          }
        }
        c if c.is_ascii() => Some(Key::Char(c as char)),
        _ => Some(Key::Unknown),
      }
    }
    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => None,
    Err(_) => Some(Key::Unknown),
  }
}

pub fn key_listener() -> Receiver<Key> {
  let (tx, rx): (Sender<Key>, Receiver<Key>) = channel();

  thread::spawn(move || {
    loop {
      if let Some(key) = read_key() {
        if tx.send(key).is_err() {
          break;
        }
      }
      thread::sleep(time::Duration::from_millis(CAPTURING_RATE));
    }
  });
  rx
}
