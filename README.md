# rustui

A modern, safe, and ergonomic terminal UI library for Rust, inspired by ncurses but designed with Rust's ownership model and safety guarantees in mind.

## Features

- **Memory Safe**: Built entirely in safe Rust with RAII patterns
- **Cross-platform**: Works on Unix-like systems (Linux, macOS, etc.)
- **Double Buffering**: Efficient rendering with differential updates
- **Rich Text Styling**: Support for colors, attributes (bold, italic, underline, etc.)
- **Non-blocking Input**: Asynchronous keyboard input handling
- **Thread-safe**: Multi-threaded rendering and input processing

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
rustui = "0.1.0"
```

### Basic Example

```rust
use rustui::*;
use std::{thread, time};

const RENDERING_RATE: time::Duration = time::Duration::from_millis(16);
const INPUT_CAPTURING_RATE: time::Duration = time::Duration::from_millis(10); // ms

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut win = Window::new(false)?;
    win.init()?; // Initialize the window (enable raw mode)
    win.start(RENDERING_RATE); // Start the rendering thread

    // Create a key listener
    let (mut key_listener, key_rx) = KeyListener::new(INPUT_CAPTURING_RATE);

    let x_center = win.width / 2;
    let y_center: usize = win.height / 2;

    loop {
        // Check for key presses
        if let Ok(key) = key_rx.try_recv() {
            match key {
                Key::Char('q') => break,
                _ => (),
            }
        }

        // Render the frame
        {
            let mut canvas = win.get_canvas();
            canvas.set_border(style![term::Attr::BOLD]);
            canvas.set_str(
                x_center,
                y_center,
                "Hello, world! (Press 'q' to quit)",
                style![term::Attr::ITALIC, term::Color::FgRgb(128, 255, 128)],
                Align::Center,
            );
        }

        thread::sleep(time::Duration::from_millis(1)); // Sleep to prevent high CPU usage
    }

    key_listener.stop()?; // Stop the key listener
    win.end()?; // Restore terminal state
    Ok(())
}
```

## API Overview

### Core Components

#### Window
The main interface for terminal management:
```rust
let mut window = Window::new()?;
window.init()?;                                    // Enter raw mode
let fps_rx = window.start(Duration::from_millis(16)); // Start rendering
window.end()?;                                     // Restore terminal
```

#### Framebuffer
Double-buffered screen representation:
```rust
let mut fb = window.get_lock();
fb.clear();                                        // Clear screen
fb.set_char(x, y, 'A', style![Attr::BOLD]);      // Set character
fb.set_str(x, y, "Hello", style![Color::Fg(1)]); // Set string
fb.set_border(style![Attr::NORMAL]);              // Draw border
```

#### Input Handling
Non-blocking keyboard input:
```rust
let (mut listener, key_rx) = KeyListener::new(Duration::from_millis(15));

// In your main loop
if let Ok(key) = key_rx.try_recv() {
    match key {
        Key::Char(c) => println!("Character: {}", c),
        Key::ArrowUp => println!("Up arrow pressed"),
        Key::Escape => break,
        _ => {}
    }
}

listener.stop()?; // Clean shutdown
```

#### Styling
Rich text styling with the `style!` macro:
```rust
// Attributes
style![Attr::BOLD]
style![Attr::ITALIC, Attr::UNDERLINE]

// Colors
style![Color::Fg(1)]              // Foreground color (0-15)
style![Color::Bg(4)]              // Background color
style![Color::Fg24(255)]          // 24-bit color

// Combined
style![Attr::BOLD, Color::Fg(2), Color::Bg(0)]
```

### Available Attributes
- `Attr::NORMAL` - Reset to normal
- `Attr::BOLD` - Bold text
- `Attr::THIN` - Thin/dim text
- `Attr::ITALIC` - Italic text
- `Attr::UNDERLINE` - Underlined text
- `Attr::BLINK` - Blinking text
- `Attr::INVERT` - Inverted colors
- `Attr::HIDDEN` - Hidden text
- `Attr::REMOVE` - Strikethrough

### Color Support
- **8-color mode**: `Color::Fg(0..7)` and `Color::Bg(0..7)`
- **256-color mode**: `Color::Fg(0..255)` and `Color::Bg(0..255)`
- **24-bit RGB**: `Color::FgRgb(r, g, b)` and `Color::BgRgb(r, g, b)`
- **Convenience**: `Color::Fg24(color)` for grayscale

## Example Applications

This repository includes a demo application that showcases the library's capabilities:

```bash
cargo run --example tetris
```

The demo features:
- Real-time character movement with arrow keys
- FPS display
- Various text styling examples

## Dependencies

- `nix` (0.27+) - For Unix terminal control
- `bitflags` (2.0+) - For attribute flag management
- `rand`

## Platform Support

Currently supports Unix-like systems:
- Linux
- macOS
- BSD variants

Windows support may be added in future versions.

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues for bugs and feature requests.

### Development

```bash
# Clone the repository
git clone <repository-url>

# Build the library
cargo build

# Run tests
cargo test

# Run the demo
cargo run
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

---

**Note**: This library is designed for educational purposes and as a foundation for terminal-based applications. For production use, consider established libraries like `crossterm` or `tui-rs` depending on your needs.
