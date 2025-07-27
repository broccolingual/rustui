use std::{thread, time};

pub mod term;
pub mod framebuffer;
pub mod input;
pub mod window;

fn main() {
    let mut win = window::Window::new();
    win.init();
    win.start();

    let rx = input::key_listener();

    {
      let mut back_locked = win.get_mutex_lock();
      back_locked.set_str(&4, &2, &String::from("Hello, World !"), term::Attr::Normal);
      back_locked.set_str(&4, &3, &String::from("Hello, World !"), term::Attr::Bold);
      back_locked.set_str(&4, &4, &String::from("Hello, World !"), term::Attr::Italic);
      back_locked.set_str(&4, &5, &String::from("Hello, World !"), term::Attr::Thin);
      back_locked.set_str(&4, &6, &String::from("Hello, World !"), term::Attr::Underline);
      back_locked.set_str(&4, &7, &String::from("Hello, World !"), term::Attr::ForeColor(128));
      back_locked.set_border(term::Attr::Normal);
    }

    let mut x = win.width / 2;
    let mut y = win.height / 2;

    loop {
      if let Ok(key) = rx.try_recv() {
        match key {
          input::Key::Char('q') => break,
          input::Key::ArrowUp => {
            if y > 0 { y -= 1; }
          }
          input::Key::ArrowDown => {
            if y < win.height - 1 { y += 1; }
          }
          input::Key::ArrowRight => {
            if x < win.width - 1 { x += 1; }
          }
          input::Key::ArrowLeft => {
            if x > 0 { x -= 1; }
          }
          _ => (),
        }
      }

      {
        let mut back_locked = win.get_mutex_lock();
        back_locked.clear();
        back_locked.set_border(term::Attr::Normal);
        back_locked.set_str(&(x - 1), &y, &String::from("●"), term::Attr::ForeColor(128));
        back_locked.set_str(&3, &2, &format!("(x, y) = ({}, {})", &x, &y).to_string(), term::Attr::Italic);
      }

      thread::sleep(time::Duration::from_millis(1));
    }

    win.end();
}
