use std::io::{self, Write};

use crate::term::{self, ColorExt};

#[derive(Clone, PartialEq, Debug)]
pub struct Cell {
    pub ch: char,
    pub attrs: term::Attr,
    pub fg: term::Color,
    pub bg: term::Color,
}

impl Cell {
    pub fn new() -> Self {
        Self {
            ch: ' ',
            attrs: term::Attr::NORMAL,
            fg: term::Color::new(),
            bg: term::Color::new(),
        }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::new()
    }
}

pub enum Align {
    Left,
    Center,
    Right,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<Cell>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let buffer = vec![Cell::default(); width * height];
        Self {
            width,
            height,
            buffer,
        }
    }

    /// Check whether x and y fit within the frame buffer size.
    pub fn check_range(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height
    }

    /// Initialize the buffer.
    pub fn clear(&mut self) {
        for cell in &mut self.buffer {
            *cell = Cell::default();
        }
    }

    /// Write a character and its attributes to the buffer.
    pub fn set_char(
        &mut self,
        x: usize,
        y: usize,
        ch: char,
        attrs: term::Attr,
        fg: term::Color,
        bg: term::Color,
    ) {
        if self.check_range(x, y) {
            let idx = y * self.width + x;
            self.buffer[idx].ch = ch;
            self.buffer[idx].attrs = attrs;
            self.buffer[idx].fg = fg;
            self.buffer[idx].bg = bg;
        }
    }

    /// Write a string and its attributes to the buffer.
    pub fn set_str(
        &mut self,
        x: usize,
        y: usize,
        str: &str,
        attrs: term::Attr,
        fg: term::Color,
        bg: term::Color,
        align: Align,
    ) {
        let start_x = match align {
            Align::Left => x,
            Align::Center => {
                if str.len() > x {
                    0
                } else {
                    x - str.len() / 2
                }
            }
            Align::Right => {
                if str.len() > x {
                    0
                } else {
                    x - str.len()
                }
            }
        };
        for (i, ch) in str.chars().enumerate() {
            self.set_char(start_x + i, y, ch, attrs, fg, bg);
        }
    }

    /// Draw the border around the buffer.
    pub fn set_border(&mut self, attrs: term::Attr, fg: term::Color, bg: term::Color) {
        let w = self.width;
        let h = self.height;

        // Draw the top and bottom borders
        for x in 1..w - 1 {
            self.set_char(x, 0, '─', attrs, fg, bg);
            self.set_char(x, h - 1, '─', attrs, fg, bg);
        }

        // Draw the left and right borders
        for y in 1..h - 1 {
            self.set_char(0, y, '│', attrs, fg, bg);
            self.set_char(w - 1, y, '│', attrs, fg, bg);
        }

        // Draw the corners
        self.set_char(0, 0, '┌', attrs, fg, bg);
        self.set_char(w - 1, 0, '┐', attrs, fg, bg);
        self.set_char(0, h - 1, '└', attrs, fg, bg);
        self.set_char(w - 1, h - 1, '┘', attrs, fg, bg);
    }

    /// Draw a vertical line in the buffer.
    pub fn set_vline(
        &mut self,
        x: usize,
        y_start: usize,
        y_end: usize,
        attrs: term::Attr,
        fg: term::Color,
        bg: term::Color,
    ) {
        for y in y_start..=y_end {
            self.set_char(x, y, '│', attrs, fg, bg);
        }
    }

    /// Draw a horizontal line in the buffer.
    pub fn set_hline(
        &mut self,
        y: usize,
        x_start: usize,
        x_end: usize,
        attrs: term::Attr,
        fg: term::Color,
        bg: term::Color,
    ) {
        for x in x_start..=x_end {
            self.set_char(x, y, '─', attrs, fg, bg);
        }
    }

    /// Combine the contents of the buffer.
    pub fn combine(&mut self, other: &Framebuffer, x_offset: usize, y_offset: usize) {
        for y in 0..other.height {
            for x in 0..other.width {
                if self.check_range(x + x_offset, y + y_offset) {
                    let idx = (y + y_offset) * self.width + (x + x_offset);
                    self.buffer[idx] = other.buffer[y * other.width + x].clone();
                }
            }
        }
    }

    /// Compare the back buffer and front buffer, draw the differences, and update the front buffer with the contents of the back buffer.
    pub fn refresh(&mut self, back_fb: &Framebuffer) -> io::Result<()> {
        if self.height != back_fb.height || self.width != back_fb.width {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Framebuffer sizes do not match",
            ));
        }

        let mut stdout_lock = io::stdout().lock(); // Lock standard output
        let mut prev_attrs = term::Attr::NORMAL;
        let mut prev_fg: term::Color = term::Color::new();
        let mut prev_bg: term::Color = term::Color::new();

        stdout_lock.write_all("\x1B[0m".as_bytes())?; // Reset all attributes
        stdout_lock.flush()?;

        // Collect all changed cells first
        let mut changes = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let front = &self.buffer[idx];
                let back = &back_fb.buffer[idx];

                if front != back {
                    changes.push((x, y, idx, back));
                }
            }
        }

        // Draw the output for each changed cell
        for (x, y, idx, back) in changes {
            let mut cell_output = String::new();

            cell_output.push_str(&format!("\x1B[{};{}H", y + 1, x + 1)); // Move to the target coordinates
            if prev_attrs != back.attrs {
                prev_attrs = back.attrs;
                cell_output.push_str(&back.attrs.to_ansi());
            }
            if prev_fg != back.fg {
                prev_fg = back.fg;
                if back.fg.is_valid() {
                    cell_output.push_str(&format!(
                        "\x1B[38;2;{};{};{}m",
                        back.fg.0, back.fg.1, back.fg.2
                    ));
                } else {
                    cell_output.push_str("\x1B[39m"); // Reset foreground color
                }
            }
            if prev_bg != back.bg {
                prev_bg = back.bg;
                if back.bg.is_valid() {
                    cell_output.push_str(&format!(
                        "\x1B[48;2;{};{};{}m",
                        back.bg.0, back.bg.1, back.bg.2
                    ));
                } else {
                    cell_output.push_str("\x1B[49m"); // Reset background color
                }
            }
            cell_output.push(back.ch); // Add the character

            stdout_lock.write_all(cell_output.as_bytes())?;
            stdout_lock.flush()?; // Flush to reflect the output

            self.buffer[idx] = back.clone(); // Copy the Cell to the front buffer
        }
        Ok(())
    }
}
