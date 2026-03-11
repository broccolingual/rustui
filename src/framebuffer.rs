use std::fmt::Write as FmtWrite;
use std::io::{self, Write};

use crate::{Attr, Color};

const CHUNK_SIZE: usize = 256;

/// Represents a single cell in the framebuffer.
#[derive(Clone, Copy, PartialEq, Debug)]
struct Cell {
    /// The character displayed in the cell.
    ch: char,
    /// Text attributes (bold, italic, underline, etc.)
    attrs: Attr,
    /// Foreground color as RGB values (0-255 each)
    fg: Color,
    /// Background color as RGB values (0-255 each)
    bg: Color,
}

impl Cell {
    /// Create a new cell with default values.
    ///
    /// Returns a `Cell` instance with default attributes and colors.
    fn new() -> Self {
        Self {
            ch: ' ',
            attrs: Attr::default(),
            fg: Color::default(),
            bg: Color::default(),
        }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents the horizontal alignment of text.
pub enum Align {
    Left,
    Center,
    Right,
}

/// Represents the framebuffer for rendering.
#[derive(Clone, PartialEq, Debug)]
pub struct Framebuffer {
    /// The width of the framebuffer.
    pub width: usize,
    /// The height of the framebuffer.
    pub height: usize,
    buffer: Box<[Cell]>,
}

impl Framebuffer {
    /// Create a new framebuffer with the specified width and height.
    ///
    /// * `width`: The width of the framebuffer.
    /// * `height`: The height of the framebuffer.
    ///
    /// Returns a new `Framebuffer` instance.
    pub fn new(width: usize, height: usize) -> Self {
        let buffer = vec![Cell::default(); width * height].into_boxed_slice();
        Self {
            width,
            height,
            buffer,
        }
    }

    /// Initialize the buffer.
    pub fn clear(&mut self) {
        self.buffer.fill(Cell::default());
    }

    /// Write a character and its attributes to the buffer.
    ///
    /// * `x`: The x-coordinate of the cell to modify.
    /// * `y`: The y-coordinate of the cell to modify.
    /// * `ch`: The character to display in the cell.
    /// * `attrs`: The attributes of the cell.
    /// * `fg`: The foreground color of the cell.
    /// * `bg`: The background color of the cell.
    pub fn set_char(&mut self, x: usize, y: usize, ch: char, attrs: Attr, fg: Color, bg: Color) {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            self.buffer[idx].ch = ch;
            self.buffer[idx].attrs = attrs;
            self.buffer[idx].fg = fg;
            self.buffer[idx].bg = bg;
        }
    }

    /// Write a string and its attributes to the buffer.
    ///
    /// * `x`: The x-coordinate of the cell to modify.
    /// * `y`: The y-coordinate of the cell to modify.
    /// * `str`: The string to display in the cell.
    /// * `attrs`: The attributes of the cell.
    /// * `fg`: The foreground color of the cell.
    /// * `bg`: The background color of the cell.
    /// * `align`: The alignment of the text.
    #[allow(clippy::too_many_arguments)]
    pub fn set_str(
        &mut self,
        x: usize,
        y: usize,
        str: &str,
        attrs: Attr,
        fg: Color,
        bg: Color,
        align: Align,
    ) {
        let str_len = str.chars().count();
        let start_x = match align {
            Align::Left => x,
            Align::Center => x.saturating_sub(str_len / 2),
            Align::Right => x.saturating_sub(str_len),
        };
        for (i, ch) in str.chars().enumerate() {
            let px = start_x + i;
            if px < self.width {
                self.set_char(px, y, ch, attrs, fg, bg);
            }
        }
    }

    /// Draw the border around the buffer.
    ///
    /// * `attrs`: The attributes of the border.
    /// * `fg`: The foreground color of the border.
    /// * `bg`: The background color of the border.
    pub fn set_border(&mut self, attrs: Attr, fg: Color, bg: Color) {
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
        self.set_char(0, 0, '╭', attrs, fg, bg);
        self.set_char(w - 1, 0, '╮', attrs, fg, bg);
        self.set_char(0, h - 1, '╰', attrs, fg, bg);
        self.set_char(w - 1, h - 1, '╯', attrs, fg, bg);
    }

    /// Set a named border around the buffer.
    ///
    /// * `title`: The name to display in the border.
    /// * `align`: The alignment of the name.
    /// * `attrs`: The attributes of the border.
    /// * `fg`: The foreground color of the border.
    /// * `bg`: The background color of the border.
    pub fn set_named_border(
        &mut self,
        title: &str,
        align: Align,
        attrs: Attr,
        fg: Color,
        bg: Color,
    ) {
        self.set_border(attrs, fg, bg);

        if title.chars().count() > self.width - 6 {
            let mut truncated_title = title.chars().take(self.width - 9).collect::<String>();
            truncated_title.push_str(" ...");
            truncated_title.insert(0, ' ');
            self.set_str(2, 0, &truncated_title, attrs, fg, bg, Align::Left);
            return;
        }

        let spaced_title = format!(" {title} ");
        match align {
            Align::Left => self.set_str(2, 0, &spaced_title, attrs, fg, bg, Align::Left),
            Align::Center => self.set_str(
                self.width / 2,
                0,
                &spaced_title,
                attrs,
                fg,
                bg,
                Align::Center,
            ),
            Align::Right => self.set_str(
                self.width.saturating_sub(2),
                0,
                &spaced_title,
                attrs,
                fg,
                bg,
                Align::Right,
            ),
        }
    }

    /// Draw a vertical line in the buffer.
    ///
    /// * `x`: The x-coordinate of the line.
    /// * `y_start`: The starting y-coordinate of the line.
    /// * `y_end`: The ending y-coordinate of the line.
    /// * `attrs`: The attributes of the line.
    /// * `fg`: The foreground color of the line.
    /// * `bg`: The background color of the line.
    pub fn set_vline(
        &mut self,
        x: usize,
        y_start: usize,
        y_end: usize,
        attrs: Attr,
        fg: Color,
        bg: Color,
    ) {
        for y in y_start..=y_end {
            self.set_char(x, y, '│', attrs, fg, bg);
        }
    }

    /// Draw a horizontal line in the buffer.
    ///
    /// * `y`: The y-coordinate of the line.
    /// * `x_start`: The starting x-coordinate of the line.
    /// * `x_end`: The ending x-coordinate of the line.
    /// * `attrs`: The attributes of the line.
    /// * `fg`: The foreground color of the line.
    /// * `bg`: The background color of the line.
    pub fn set_hline(
        &mut self,
        y: usize,
        x_start: usize,
        x_end: usize,
        attrs: Attr,
        fg: Color,
        bg: Color,
    ) {
        for x in x_start..=x_end {
            self.set_char(x, y, '─', attrs, fg, bg);
        }
    }

    /// Combine the contents of the buffer.
    ///
    /// * `other`: The other framebuffer to combine with.
    /// * `x_offset`: The x-coordinate offset to start combining.
    /// * `y_offset`: The y-coordinate offset to start combining.
    pub fn combine(&mut self, other: &Framebuffer, x_offset: usize, y_offset: usize) {
        let max_y = other.height.min(self.height.saturating_sub(y_offset));
        let max_x = other.width.min(self.width.saturating_sub(x_offset));
        if max_x == 0 || max_y == 0 {
            return;
        }
        for y in 0..max_y {
            let dst_start = (y + y_offset) * self.width + x_offset;
            let src_start = y * other.width;
            self.buffer[dst_start..dst_start + max_x]
                .copy_from_slice(&other.buffer[src_start..src_start + max_x]);
        }
    }

    /// Set the foreground color for all cells in the buffer.
    ///
    /// * `color`: The color to set.
    pub fn set_fg_color(&mut self, color: Color) {
        self.buffer.iter_mut().for_each(|cell| cell.fg = color);
    }

    /// Set the background color for all cells in the buffer.
    ///
    /// * `color`: The color to set.
    pub fn set_bg_color(&mut self, color: Color) {
        self.buffer.iter_mut().for_each(|cell| cell.bg = color);
    }

    /// Compare the back buffer and front buffer, draw the differences, and update the front buffer with the contents of the back buffer.
    ///
    /// * `back_fb`: The back buffer to compare against.
    ///
    /// Returns `Ok(())` if successful, or an error if the framebuffers do not match.
    pub fn refresh(&mut self, back_fb: &Framebuffer) -> io::Result<()> {
        if self.height != back_fb.height || self.width != back_fb.width {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Framebuffer sizes do not match",
            ));
        }

        let mut stdout_lock = io::stdout().lock(); // Lock standard output
        let mut prev_attrs = Attr::NORMAL;
        let mut prev_fg: Color = Color::default();
        let mut prev_bg: Color = Color::default();
        let mut has_changes = false;

        let mut chunk = String::with_capacity(CHUNK_SIZE);

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let back = back_fb.buffer[idx]; // Cell is Copy; no heap allocation

                if self.buffer[idx] != back {
                    if !has_changes {
                        chunk.push_str("\x1B[0m"); // Reset all attributes before first change
                        has_changes = true;
                    }
                    write!(chunk, "\x1B[{};{}H", y + 1, x + 1).unwrap(); // Move to the target coordinates
                    if prev_attrs != back.attrs {
                        prev_attrs = back.attrs;
                        back.attrs.write_ansi(&mut chunk);
                    }
                    if prev_fg != back.fg {
                        prev_fg = back.fg;
                        back.fg.write_ansi(true, &mut chunk);
                    }
                    if prev_bg != back.bg {
                        prev_bg = back.bg;
                        back.bg.write_ansi(false, &mut chunk);
                    }
                    chunk.push(back.ch); // Add the character
                    self.buffer[idx] = back; // Copy the Cell to the front buffer

                    if chunk.len() >= CHUNK_SIZE {
                        stdout_lock.write_all(chunk.as_bytes())?;
                        stdout_lock.flush()?;
                        chunk.clear();
                    }
                }
            }
        }
        if !chunk.is_empty() {
            stdout_lock.write_all(chunk.as_bytes())?;
            stdout_lock.flush()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_init() {
        let cell = Cell::new();
        assert_eq!(cell.ch, ' ');
        assert_eq!(cell.attrs, Attr::NORMAL);
        assert_eq!(cell.bg, Color::default());
        assert_eq!(cell.fg, Color::default());
    }

    #[test]
    fn test_fb_init() {
        let fb = Framebuffer::new(4, 2);
        assert_eq!(fb.width, 4);
        assert_eq!(fb.height, 2);
        assert_eq!(fb.buffer.len(), 8);
    }

    #[test]
    fn test_fb_clear() {
        let mut fb = Framebuffer::new(4, 2);
        fb.set_char(
            0,
            0,
            'X',
            Attr::default(),
            Color::default(),
            Color::default(),
        );
        fb.clear();
        for cell in fb.buffer {
            assert_eq!(cell.ch, ' ');
            assert_eq!(cell.attrs, Attr::default());
            assert_eq!(cell.bg, Color::default());
            assert_eq!(cell.fg, Color::default());
        }
    }

    #[test]
    fn test_fb_set_char() {
        let mut fb = Framebuffer::new(4, 2);
        fb.set_char(
            0,
            0,
            'X',
            Attr::default(),
            Color::default(),
            Color::default(),
        );
        assert_eq!(fb.buffer[0].ch, 'X');
        assert_eq!(fb.buffer[0].attrs, Attr::default());
        assert_eq!(fb.buffer[0].bg, Color::default());
        assert_eq!(fb.buffer[0].fg, Color::default());
    }

    #[test]
    fn test_fb_set_str() {
        let mut fb = Framebuffer::new(4, 2);
        fb.set_str(
            0,
            0,
            "XY",
            Attr::default(),
            Color::default(),
            Color::default(),
            Align::Left,
        );
        assert_eq!(fb.buffer[0].ch, 'X');
        assert_eq!(fb.buffer[1].ch, 'Y');
    }

    #[test]
    fn test_fb_set_border() {
        let mut fb = Framebuffer::new(4, 3);
        fb.set_border(Attr::default(), Color::White, Color::default());
        assert_eq!(fb.buffer[0].ch, '╭');
        assert_eq!(fb.buffer[1].ch, '─');
        assert_eq!(fb.buffer[2].ch, '─');
        assert_eq!(fb.buffer[3].ch, '╮');
        assert_eq!(fb.buffer[4].ch, '│');
        assert_eq!(fb.buffer[5].ch, ' ');
        assert_eq!(fb.buffer[6].ch, ' ');
        assert_eq!(fb.buffer[7].ch, '│');
        assert_eq!(fb.buffer[8].ch, '╰');
        assert_eq!(fb.buffer[9].ch, '─');
        assert_eq!(fb.buffer[10].ch, '─');
        assert_eq!(fb.buffer[11].ch, '╯');
    }

    #[test]
    fn test_fb_set_named_border() {}

    #[test]
    fn test_fb_set_vline() {
        let mut fb = Framebuffer::new(4, 3);
        fb.set_vline(1, 0, 3, Attr::default(), Color::default(), Color::default());
        assert_eq!(fb.buffer[1].ch, '│');
        assert_eq!(fb.buffer[5].ch, '│');
        assert_eq!(fb.buffer[9].ch, '│');
    }

    #[test]
    fn test_fb_set_hline() {
        let mut fb = Framebuffer::new(4, 3);
        fb.set_hline(1, 0, 4, Attr::default(), Color::default(), Color::default());
        assert_eq!(fb.buffer[4].ch, '─');
        assert_eq!(fb.buffer[5].ch, '─');
        assert_eq!(fb.buffer[6].ch, '─');
        assert_eq!(fb.buffer[7].ch, '─');
    }

    #[test]
    fn test_fb_set_fg_color() {
        let mut fb = Framebuffer::new(4, 2);
        fb.set_fg_color(Color::Red);
        for cell in &fb.buffer {
            assert_eq!(cell.fg, Color::Red);
        }
    }

    #[test]
    fn test_fb_set_bg_color() {
        let mut fb = Framebuffer::new(4, 2);
        fb.set_bg_color(Color::Blue);
        for cell in &fb.buffer {
            assert_eq!(cell.bg, Color::Blue);
        }
    }

    #[test]
    fn test_fb_combine() {
        let mut fb1 = Framebuffer::new(3, 3);
        fb1.set_border(Attr::default(), Color::default(), Color::default());
        let mut fb2 = Framebuffer::new(1, 1);
        fb2.set_str(
            0,
            0,
            "A",
            Attr::BOLD,
            Color::default(),
            Color::default(),
            Align::Left,
        );
        fb1.combine(&fb2, 1, 1);
        assert_eq!(fb1.buffer[0].ch, '╭');
        assert_eq!(fb1.buffer[1].ch, '─');
        assert_eq!(fb1.buffer[2].ch, '╮');
        assert_eq!(fb1.buffer[3].ch, '│');
        assert_eq!(fb1.buffer[4].ch, 'A');
        assert_eq!(fb1.buffer[4].attrs, Attr::BOLD);
        assert_eq!(fb1.buffer[5].ch, '│');
        assert_eq!(fb1.buffer[6].ch, '╰');
        assert_eq!(fb1.buffer[7].ch, '─');
        assert_eq!(fb1.buffer[8].ch, '╯');
    }

    #[test]
    fn test_fb_combine_out_of_bounds() {
        let mut fb1 = Framebuffer::new(3, 2);
        let mut fb2 = Framebuffer::new(2, 2);
        fb2.set_char(
            0,
            0,
            'X',
            Attr::default(),
            Color::default(),
            Color::default(),
        );

        // x_offset fully outside: must not panic
        fb1.combine(&fb2, 7, 0);
        // y_offset fully outside: must not panic
        fb1.combine(&fb2, 0, 5);
        // both offsets outside
        fb1.combine(&fb2, 7, 5);
        // fb1 should be unchanged (all spaces)
        for cell in &fb1.buffer {
            assert_eq!(cell.ch, ' ');
        }
    }
}
