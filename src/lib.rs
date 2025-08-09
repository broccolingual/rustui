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
