/*!
# rustui

The simplest terminal UI library designed for Rust. Developed with Rust's ownership model and safety guarantees in mind.

## Features

- **Cross-platform**: Works on Unix-like systems (Linux, macOS, etc.)
- **Double Buffering**: Efficient rendering with differential updates
- **Rich Text Styling**: Support for colors, attributes (bold, italic, underline, etc.)
- **Non-blocking Input**: Asynchronous keyboard input handling
- **Thread-safe**: Multi-threaded rendering and input processing

## Architecture

The library is organized into several core modules:

- [`framebuffer`] - Character grid for efficient rendering
- [`window`] - High-level windowing abstraction with thread management
- [`input`] - Non-blocking keyboard and mouse input handling
- [`term`] - Terminal colors, attributes, and ANSI escape sequences
- [`render`] - Rendering utilities and drawing primitives

## Performance

rustui is optimized for performance with:
- Differential updates (only changed cells are redrawn)
- Efficient ANSI sequence generation

*/

/// A module for handling the framebuffer.
pub mod framebuffer;
/// A module for handling user input.
pub mod input;
/// A module for a rendering context.
pub mod render;
/// A module for handling terminal colors and attributes.
pub mod term;
/// A module for handling windowing.
pub mod window;

pub use framebuffer::*;
pub use input::*;
pub use render::*;
pub use term::*;
pub use window::*;
