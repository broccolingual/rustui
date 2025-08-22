use std::io::{self, Read, StdinLock};
use std::sync::mpsc::{self, Receiver, Sender, SyncSender};
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
    Ctrl(char),
    Enter, // Ctrl + M
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
    Unknown,
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

    let end_marker = buf[n - 1];
    let is_press = end_marker == b'M';
    let is_release = end_marker == b'm';

    if !is_press && !is_release {
        return None;
    }

    let mut values = [0u16; 3];
    let mut value_idx = 0;
    let mut current_value = 0u16;

    for &byte in &buf[3..n - 1] {
        if byte == b';' {
            if value_idx >= 3 {
                return None;
            }
            values[value_idx] = current_value;
            value_idx += 1;
            current_value = 0;
        } else if byte.is_ascii_digit() {
            current_value = current_value * 10 + (byte - b'0') as u16;
        } else {
            return None;
        }
    }

    if value_idx != 2 {
        return None;
    }
    values[2] = current_value;

    let button_code = values[0] as u8;
    let x = values[1];
    let y = values[2];

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
/// * `buf` - The buffer containing the escape sequence.
/// * `n` - The length of the buffer.
///
/// Returns `InputEvent` corresponding to the escape sequence.
fn parse_escape_sequence(buf: &[u8], n: usize) -> InputEvent {
    if n < 3 {
        return InputEvent::Key(Key::Escape);
    }

    // We can check this using `showkey -a` command
    let e = match &buf[1] {
        // Check SS3 sequence
        b'O' => {
            let key = match buf[2] {
                b'P' => Key::F1,
                b'Q' => Key::F2,
                b'R' => Key::F3,
                b'S' => Key::F4,
                _ => Key::Unknown,
            };
            InputEvent::Key(key)
        }
        // Check CSI sequence
        b'[' => {
            if buf[2] == b'<' {
                // Check for mouse event (SGR format)
                if let Some(mouse_event) = parse_mouse_event(buf, n) {
                    InputEvent::Mouse(mouse_event)
                } else {
                    InputEvent::Unknown
                }
            } else {
                // Check special keys
                let key = match &buf[2..] {
                    [b'A', ..] => Key::ArrowUp,
                    [b'B', ..] => Key::ArrowDown,
                    [b'C', ..] => Key::ArrowRight,
                    [b'D', ..] => Key::ArrowLeft,
                    [b'1', b'~', ..] | [b'H', ..] => Key::Home,
                    [b'2', b'~', ..] => Key::Insert,
                    [b'3', b'~', ..] => Key::Delete,
                    [b'5', b'~', ..] => Key::PageUp,
                    [b'6', b'~', ..] => Key::PageDown,
                    [b'4', b'~', ..] | [b'7', b'~', ..] | [b'F', ..] => Key::End,
                    [b'1', b'0', b'~', ..] => Key::F0,
                    [b'1', b'1', b'~', ..] => Key::F1,
                    [b'1', b'2', b'~', ..] => Key::F2,
                    [b'1', b'3', b'~', ..] => Key::F3,
                    [b'1', b'4', b'~', ..] => Key::F4,
                    [b'1', b'5', b'~', ..] => Key::F5,
                    [b'1', b'7', b'~', ..] => Key::F6,
                    [b'1', b'8', b'~', ..] => Key::F7,
                    [b'1', b'9', b'~', ..] => Key::F8,
                    [b'2', b'0', b'~', ..] => Key::F9,
                    [b'2', b'1', b'~', ..] => Key::F10,
                    [b'2', b'3', b'~', ..] => Key::F11,
                    [b'2', b'4', b'~', ..] => Key::F12,
                    [b'2', b'5', b'~', ..] => Key::F13,
                    [b'2', b'6', b'~', ..] => Key::F14,
                    [b'2', b'8', b'~', ..] => Key::F15,
                    [b'2', b'9', b'~', ..] => Key::F16,
                    [b'3', b'1', b'~', ..] => Key::F17,
                    [b'3', b'2', b'~', ..] => Key::F18,
                    [b'3', b'3', b'~', ..] => Key::F19,
                    [b'3', b'4', b'~', ..] => Key::F20,
                    _ => Key::Unknown,
                };
                InputEvent::Key(key)
            }
        }
        _ => InputEvent::Unknown,
    };
    e
}

/// Read input (key or mouse) from standard input.
///
/// * `stdin` - The standard input stream.
/// * `buf` - The buffer to read the input into.
///
/// Returns `Some(InputEvent)` if an event is detected, `None` otherwise.
fn read_key(stdin: &mut StdinLock, buf: &mut [u8]) -> io::Result<Option<InputEvent>> {
    match stdin.read(buf) {
        Ok(0) => Ok(None),
        Ok(n) => {
            let event = match buf[0] {
                0x01..=0x1A => {
                    let c = ((buf[0] - 0x01) + b'a') as char;
                    if c == 'm' {
                        InputEvent::Key(Key::Enter) // Ctrl + M
                    } else {
                        InputEvent::Key(Key::Ctrl(c))
                    }
                }
                0x1B => parse_escape_sequence(buf, n),
                0x20..=0x7e => InputEvent::Key(Key::Char(buf[0] as char)),
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
    handle: Option<thread::JoinHandle<()>>,
    /// The stop signal sender for the input listener.
    stop_signal: Option<Sender<()>>,
    /// The receiver for input events.
    input_rx: Receiver<InputEvent>,
}

impl InputListener {
    /// Create a new input listener that listens for key events at a specified rate.
    ///
    /// * `rate` - The rate at which to poll for input events.
    ///
    /// Returns `InputListener` instance.
    pub fn new(rate: Duration) -> Self {
        let (input_tx, input_rx): (SyncSender<InputEvent>, Receiver<InputEvent>) =
            mpsc::sync_channel(256);
        let (stop_tx, stop_rx): (Sender<()>, Receiver<()>) = mpsc::channel();

        let handle = thread::spawn(move || {
            let mut stdin = io::stdin().lock();
            let mut buf = [0u8; 64];

            loop {
                if stop_rx.try_recv().is_ok() {
                    break; // Stop the loop if a stop signal is received
                }

                match read_key(&mut stdin, &mut buf) {
                    Ok(Some(event)) => {
                        match input_tx.try_send(event) {
                            Ok(_) => {}
                            Err(mpsc::TrySendError::Full(_)) => {} // If the channel is full, we can drop the event
                            Err(mpsc::TrySendError::Disconnected(_)) => break, // Stop the loop if the receiver is dropped
                        }
                    }
                    Ok(None) => {} // No input
                    Err(_) => {}   // Read error, continue
                }
                thread::sleep(rate);
            }
        });
        Self {
            handle: Some(handle),
            stop_signal: Some(stop_tx),
            input_rx,
        }
    }

    /// Try to receive an input event.
    ///
    /// Returns `Ok(InputEvent)` if an event is received, or an error if it fails.
    pub fn try_recv(&self) -> Result<InputEvent, mpsc::TryRecvError> {
        self.input_rx.try_recv()
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
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_is_arrow() {
        // Arrow keys
        assert!(Key::ArrowUp.is_arrow());
        assert!(Key::ArrowDown.is_arrow());
        assert!(Key::ArrowLeft.is_arrow());
        assert!(Key::ArrowRight.is_arrow());

        // Not arrow keys
        assert!(!Key::Home.is_arrow());
        assert!(!Key::Char('a').is_arrow());
        assert!(!Key::F1.is_arrow());
        assert!(!Key::Unknown.is_arrow());
    }

    #[test]
    fn test_key_is_special() {
        // Special keys
        assert!(Key::ArrowUp.is_special());
        assert!(Key::Home.is_special());
        assert!(Key::End.is_special());
        assert!(Key::F1.is_special());
        assert!(Key::PageUp.is_special());
        assert!(Key::Delete.is_special());
        assert!(Key::Escape.is_special());

        // Not special keys
        assert!(!Key::Char('a').is_special());
        assert!(!Key::Char(' ').is_special());
        assert!(!Key::Char('1').is_special());
        assert!(!Key::Unknown.is_special());
    }

    #[test]
    fn test_key_is_printable() {
        // Printable characters
        assert!(Key::Char('a').is_printable());
        assert!(Key::Char('Z').is_printable());
        assert!(Key::Char('1').is_printable());
        assert!(Key::Char(' ').is_printable());
        assert!(Key::Char('!').is_printable());

        // Non-printable
        assert!(!Key::Char('\n').is_printable());
        assert!(!Key::Char('\t').is_printable());
        assert!(!Key::ArrowUp.is_printable());
        assert!(!Key::F1.is_printable());
        assert!(!Key::Unknown.is_printable());
    }

    #[test]
    fn test_parse_mouse_event_press() {
        // Left mouse button press at (10, 20): \x1B[<0;10;20M
        let buf = b"\x1B[<0;10;20M";
        let result = parse_mouse_event(buf, buf.len());

        assert_eq!(
            result,
            Some(MouseEvent::Press {
                button: MouseButton::Left,
                x: 10,
                y: 20
            })
        );
    }

    #[test]
    fn test_parse_mouse_event_release() {
        // Left mouse button release at (15, 25): \x1B[<0;15;25m
        let buf = b"\x1B[<0;15;25m";
        let result = parse_mouse_event(buf, buf.len());

        assert_eq!(
            result,
            Some(MouseEvent::Release {
                button: MouseButton::Left,
                x: 15,
                y: 25
            })
        );
    }

    #[test]
    fn test_parse_mouse_event_wheel() {
        // Wheel up: \x1B[<64;5;5M
        let buf = b"\x1B[<64;5;5M";
        let result = parse_mouse_event(buf, buf.len());

        assert_eq!(
            result,
            Some(MouseEvent::Press {
                button: MouseButton::WheelUp,
                x: 5,
                y: 5
            })
        );

        // Wheel down: \x1B[<65;5;5M
        let buf = b"\x1B[<65;5;5M";
        let result = parse_mouse_event(buf, buf.len());

        assert_eq!(
            result,
            Some(MouseEvent::Press {
                button: MouseButton::WheelDown,
                x: 5,
                y: 5
            })
        );
    }

    #[test]
    fn test_parse_mouse_event_invalid() {
        // Too short
        let buf = b"\x1B[<0";
        assert_eq!(parse_mouse_event(buf, buf.len()), None);

        // Wrong prefix
        let buf = b"\x1B[0;10;20M";
        assert_eq!(parse_mouse_event(buf, buf.len()), None);

        // Invalid end marker
        let buf = b"\x1B[<0;10;20X";
        assert_eq!(parse_mouse_event(buf, buf.len()), None);

        // Invalid format
        let buf = b"\x1B[<0;abc;20M";
        assert_eq!(parse_mouse_event(buf, buf.len()), None);
    }

    #[test]
    fn test_parse_escape_sequence_arrow_keys() {
        // Arrow keys
        assert_eq!(
            parse_escape_sequence(b"\x1B[A", 3),
            InputEvent::Key(Key::ArrowUp)
        );
        assert_eq!(
            parse_escape_sequence(b"\x1B[B", 3),
            InputEvent::Key(Key::ArrowDown)
        );
        assert_eq!(
            parse_escape_sequence(b"\x1B[C", 3),
            InputEvent::Key(Key::ArrowRight)
        );
        assert_eq!(
            parse_escape_sequence(b"\x1B[D", 3),
            InputEvent::Key(Key::ArrowLeft)
        );
    }

    #[test]
    fn test_parse_escape_sequence_function_keys() {
        // Function keys
        assert_eq!(
            parse_escape_sequence(b"\x1B[10~", 5),
            InputEvent::Key(Key::F0)
        );
        assert_eq!(
            parse_escape_sequence(b"\x1B[11~", 5),
            InputEvent::Key(Key::F1)
        );
        assert_eq!(
            parse_escape_sequence(b"\x1BOP", 4),
            InputEvent::Key(Key::F1)
        );
    }

    #[test]
    fn test_parse_escape_sequence_navigation_keys() {
        // Home key variants
        assert_eq!(
            parse_escape_sequence(b"\x1B[H", 3),
            InputEvent::Key(Key::Home)
        );
        assert_eq!(
            parse_escape_sequence(b"\x1B[1~", 4),
            InputEvent::Key(Key::Home)
        );

        // End key variants
        assert_eq!(
            parse_escape_sequence(b"\x1B[F", 3),
            InputEvent::Key(Key::End)
        );
        assert_eq!(
            parse_escape_sequence(b"\x1B[4~", 4),
            InputEvent::Key(Key::End)
        );

        // Other navigation keys
        assert_eq!(
            parse_escape_sequence(b"\x1B[2~", 4),
            InputEvent::Key(Key::Insert)
        );
        assert_eq!(
            parse_escape_sequence(b"\x1B[3~", 4),
            InputEvent::Key(Key::Delete)
        );
        assert_eq!(
            parse_escape_sequence(b"\x1B[5~", 4),
            InputEvent::Key(Key::PageUp)
        );
        assert_eq!(
            parse_escape_sequence(b"\x1B[6~", 4),
            InputEvent::Key(Key::PageDown)
        );
    }

    #[test]
    fn test_parse_escape_sequence_invalid() {
        // Too short sequence should return Escape
        assert_eq!(
            parse_escape_sequence(b"\x1B", 1),
            InputEvent::Key(Key::Escape)
        );
        assert_eq!(
            parse_escape_sequence(b"\x1B[", 2),
            InputEvent::Key(Key::Escape)
        );

        // Unknown sequence should return Unknown
        assert_eq!(
            parse_escape_sequence(b"\x1B[999~", 6),
            InputEvent::Key(Key::Unknown)
        );
    }
}
