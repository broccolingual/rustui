use std::io::{self, Read};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

/// Keyboard key events
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Key {
    ArrowUp,
    ArrowDown,
    ArrowRight,
    ArrowLeft,
    Home,
    End,
    Insert,
    Delete,
    PageUp,
    PageDown,
    Char(char),
    Escape,
    F0,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    Unknown,
}

/// Mouse button events
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    WheelUp,
    WheelDown,
}

/// Mouse event
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseEvent {
    Press { button: MouseButton, x: u16, y: u16 },
    Release { button: MouseButton, x: u16, y: u16 },
    Move { x: u16, y: u16 },
}

/// Input event
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputEvent {
    Key(Key),
    Mouse(MouseEvent),
}

impl Key {
    /// Check whether the key is an arrow key.
    ///
    /// Returns `true` if the key is an arrow key, `false` otherwise.
    pub fn is_arrow(&self) -> bool {
        matches!(
            self,
            Key::ArrowUp | Key::ArrowDown | Key::ArrowLeft | Key::ArrowRight
        )
    }

    /// Check whether the key is a special key (arrow keys, Home/End, etc.).
    ///
    /// Returns `true` if the key is a special key, `false` otherwise.
    pub fn is_special(&self) -> bool {
        !matches!(self, Key::Char(_) | Key::Unknown)
    }

    /// Check whether the key is a ASCII character.
    ///
    /// Returns `true` if the key is a ASCII character, `false` otherwise.
    pub fn is_printable(&self) -> bool {
        matches!(self, Key::Char(c) if c.is_ascii_graphic() || *c == ' ')
    }
}

/// Parse mouse event from SGR format: \x1B[<b;x;yM or \x1B[<b;x;ym
///
/// * `buf` - The buffer containing the mouse event data.
/// * `n` - The length of the content.
///
/// Returns `Some(MouseEvent)` if the event is valid, `None` otherwise.
fn parse_mouse_event(buf: &[u8], n: usize) -> Option<MouseEvent> {
    if n < 6 || buf[0] != 0x1B || buf[1] != b'[' || buf[2] != b'<' {
        return None;
    }

    let data = std::str::from_utf8(&buf[3..n]).ok()?;
    let is_press = data.ends_with('M');
    let is_release = data.ends_with('m');

    if !is_press && !is_release {
        return None;
    }

    let coords = &data[..data.len() - 1]; // Remove M or m
    let parts: Vec<&str> = coords.split(';').collect();

    if parts.len() != 3 {
        return None;
    }

    let button_code: u8 = parts[0].parse().ok()?;
    let x: u16 = parts[1].parse().ok()?;
    let y: u16 = parts[2].parse().ok()?;

    let button = match button_code & 0x03 {
        0 => MouseButton::Left,
        1 => MouseButton::Middle,
        2 => MouseButton::Right,
        _ => return None,
    };

    // Check for wheel events
    if button_code & 0x40 != 0 {
        let wheel_button = if button_code & 0x01 == 0 {
            MouseButton::WheelUp
        } else {
            MouseButton::WheelDown
        };
        return Some(MouseEvent::Press {
            button: wheel_button,
            x,
            y,
        });
    }

    if is_press {
        Some(MouseEvent::Press { button, x, y })
    } else {
        Some(MouseEvent::Release { button, x, y })
    }
}

/// Parse the escape sequence and return the corresponding Key or Mouse event.
///
/// Returns `InputEvent::Key(Key::Escape)` if the sequence is invalid.
fn parse_escape_sequence(buf: &[u8], n: usize) -> InputEvent {
    if n < 3 {
        return InputEvent::Key(Key::Escape);
    }

    // Check for mouse event (SGR format)
    if n >= 6 && buf[2] == b'<' {
        if let Some(mouse_event) = parse_mouse_event(buf, n) {
            return InputEvent::Mouse(mouse_event);
        }
    }

    let key = match &buf[1..] {
        [b'[', b'A', ..] => Key::ArrowUp,
        [b'[', b'B', ..] => Key::ArrowDown,
        [b'[', b'C', ..] => Key::ArrowRight,
        [b'[', b'D', ..] => Key::ArrowLeft,
        [b'[', b'1', b'~', ..] | [b'[', b'H', ..] => Key::Home,
        [b'[', b'2', b'~', ..] => Key::Insert,
        [b'[', b'3', b'~', ..] => Key::Delete,
        [b'[', b'5', b'~', ..] => Key::PageUp,
        [b'[', b'6', b'~', ..] => Key::PageDown,
        [b'[', b'4', b'~', ..] | [b'[', b'7', b'~', ..] | [b'[', b'F', ..] => Key::End,
        [b'[', b'1', b'0', b'~', ..] => Key::F0,
        [b'[', b'1', b'1', b'~', ..] | [b'[', b'1', b'P', ..] => Key::F1,
        [b'[', b'1', b'2', b'~', ..] | [b'[', b'1', b'Q', ..] => Key::F2,
        [b'[', b'1', b'3', b'~', ..] | [b'[', b'1', b'R', ..] => Key::F3,
        [b'[', b'1', b'4', b'~', ..] | [b'[', b'1', b'S', ..] => Key::F4,
        [b'[', b'1', b'5', b'~', ..] => Key::F5,
        [b'[', b'1', b'7', b'~', ..] => Key::F6,
        [b'[', b'1', b'8', b'~', ..] => Key::F7,
        [b'[', b'1', b'9', b'~', ..] => Key::F8,
        [b'[', b'2', b'0', b'~', ..] => Key::F9,
        [b'[', b'2', b'1', b'~', ..] => Key::F10,
        [b'[', b'2', b'3', b'~', ..] => Key::F11,
        [b'[', b'2', b'4', b'~', ..] => Key::F12,
        [b'[', b'2', b'5', b'~', ..] => Key::F13,
        [b'[', b'2', b'6', b'~', ..] => Key::F14,
        [b'[', b'2', b'8', b'~', ..] => Key::F15,
        [b'[', b'2', b'9', b'~', ..] => Key::F16,
        [b'[', b'3', b'1', b'~', ..] => Key::F17,
        [b'[', b'3', b'2', b'~', ..] => Key::F18,
        [b'[', b'3', b'3', b'~', ..] => Key::F19,
        [b'[', b'3', b'4', b'~', ..] => Key::F20,
        _ => Key::Unknown,
    };
    InputEvent::Key(key)
}

/// Read input (key or mouse) from standard input.
///
/// Returns `Some(InputEvent)` if an event is detected, `None` otherwise.
fn read_key() -> io::Result<Option<InputEvent>> {
    let mut stdin = io::stdin().lock();
    let mut buf = [0u8; 32];

    match stdin.read(&mut buf) {
        Ok(0) => Ok(None),
        Ok(n) => {
            let event = match buf[0] {
                0x1B => parse_escape_sequence(&buf, n),
                c if c.is_ascii() => InputEvent::Key(Key::Char(c as char)),
                _ => InputEvent::Key(Key::Unknown),
            };
            Ok(Some(event))
        }
        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => Ok(None),
        Err(e) => Err(e),
    }
}

/// Represents an input listener for receiving input events.
pub struct InputListener {
    /// The thread handle for the input listener.
    pub handle: Option<thread::JoinHandle<()>>,
    /// The stop signal sender for the input listener.
    pub stop_signal: Option<Sender<()>>,
}

impl InputListener {
    /// Create a new input listener that listens for key events at a specified rate.
    ///
    /// * `rate` - The rate at which to poll for input events.
    ///
    /// Returns a receiver for the input events.
    pub fn new(rate: Duration) -> Receiver<InputEvent> {
        let (input_tx, input_rx): (Sender<InputEvent>, Receiver<InputEvent>) = mpsc::channel();
        let (_, stop_rx): (Sender<()>, Receiver<()>) = mpsc::channel();

        let _ = thread::spawn(move || {
            loop {
                if stop_rx.try_recv().is_ok() {
                    break; // Stop the loop if a stop signal is received
                }

                match read_key() {
                    Ok(Some(event)) => {
                        if input_tx.send(event).is_err() {
                            break; // Stop the loop if the receiver is dropped
                        }
                    }
                    Ok(None) => {} // No input
                    Err(_) => {}   // Read error, continue
                }
                thread::sleep(rate);
            }
        });
        input_rx
    }

    /// Stop the input listener thread
    ///
    /// Returns `Ok(())` if the listener was stopped successfully, or an error if it failed.
    pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(tx) = self.stop_signal.take() {
            tx.send(())?; // Send stop signal
        }
        if let Some(handle) = self.handle.take() {
            handle
                .join()
                .map_err(|_| "Failed to join key listener thread")?; // Wait for the thread to finish
        }
        Ok(())
    }
}

impl Drop for InputListener {
    fn drop(&mut self) {
        if let Err(e) = self.stop() {
            eprintln!("Error stopping input listener: {e}");
        }
    }
}
