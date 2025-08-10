# rustui

A modern, safe, and ergonomic terminal UI library for Rust, designed with Rust's ownership model and safety guarantees in mind.

## Features

- **Cross-platform**: Works on Unix-like systems (Linux, macOS, etc.)
- **Double Buffering**: Efficient rendering with differential updates
- **Rich Text Styling**: Support for colors, attributes (bold, italic, underline, etc.)
- **Non-blocking Input**: Asynchronous keyboard input handling
- **Thread-safe**: Multi-threaded rendering and input processing

## Quick Start

```rust
use rustui::*;
use std::{thread, time};

const RENDERING_RATE: time::Duration = time::Duration::from_millis(16); // ms
const INPUT_CAPTURING_RATE: time::Duration = time::Duration::from_millis(10); // ms

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut win = Window::new(false)?;
    win.initialize(RENDERING_RATE)?; // Initialize the window and start the rendering thread
    let key_rx = KeyListener::new(INPUT_CAPTURING_RATE); // Create a key listener

    let x_center = win.width / 2;
    let y_center = win.height / 2;

    loop {
        // Check for key presses
        if let Ok(key) = key_rx.try_recv() {
            match key {
                Key::Char('q') => break,
                _ => (),
            }
        }

        // Draw the frame
        win.draw(|canvas| {
            canvas.set_border(Attr::NORMAL, (255, 255, 255), Color::new()); // Set border
            canvas.set_str(
                x_center,
                y_center,
                "Hello, world! (Press 'q' to quit)",
                Attr::NORMAL,    // Set text decoration
                (128, 255, 128), // Set text color
                (64, 64, 64),    // Set background color
                Align::Center,   // Set text alignment to center
            );
        });

        thread::sleep(time::Duration::from_millis(100)); // Sleep to prevent high CPU usage
    }
    Ok(())
}
```

## Example Applications

This repository includes a demo application that showcases the library's capabilities:

#### Hello World

```bash
cargo run --example hello_world
```

#### Colors

```bash
cargo run --example colors
```

#### Inputs

```bash
cargo run --example inputs
```

#### Tetris

```bash
cargo run --example tetris
```

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
cd rustui

# Build the library
cargo build

# Run tests
cargo test

# Run the demo
cargo run --example hello_world
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

---

**Note**: This library is designed for educational purposes and as a foundation for terminal-based applications. For production use, consider established libraries like `crossterm` or `ratatui-rs` depending on your needs.
