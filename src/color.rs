/// Represents a color in the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    RGB(u8, u8, u8),
    HSV(u8, u8, u8),
    #[default]
    None,
}

impl Color {
    /// Write ANSI escape code directly into an existing buffer, avoiding allocation.
    pub fn write_ansi(&self, fg: bool, buf: &mut String) {
        use std::fmt::Write;
        buf.push_str("\x1B[");

        match self {
            Color::Black => buf.push_str(if fg { "30m" } else { "40m" }),
            Color::Red => buf.push_str(if fg { "31m" } else { "41m" }),
            Color::Green => buf.push_str(if fg { "32m" } else { "42m" }),
            Color::Yellow => buf.push_str(if fg { "33m" } else { "43m" }),
            Color::Blue => buf.push_str(if fg { "34m" } else { "44m" }),
            Color::Magenta => buf.push_str(if fg { "35m" } else { "45m" }),
            Color::Cyan => buf.push_str(if fg { "36m" } else { "46m" }),
            Color::White => buf.push_str(if fg { "37m" } else { "47m" }),
            Color::RGB(r, g, b) => {
                let _ = write!(buf, "{};2;{};{};{}m", if fg { "38" } else { "48" }, r, g, b);
            }
            Color::HSV(h, s, v) => {
                let h = *h as u32 * 360 / 255;
                let s = *s as u32;
                let v = *v as u32;
                let c = v * s / 255;
                let h_mod = (h % 120) as i32 - 60;
                let x = c * (60 - h_mod.unsigned_abs()) / 60;
                let m = v - c;
                let (r, g, b) = match h {
                    0..=59 => (c, x, 0),
                    60..=119 => (x, c, 0),
                    120..=179 => (0, c, x),
                    180..=239 => (0, x, c),
                    240..=299 => (x, 0, c),
                    _ => (c, 0, x),
                };
                let r = (r + m).min(255);
                let g = (g + m).min(255);
                let b = (b + m).min(255);
                let _ = write!(buf, "{};2;{};{};{}m", if fg { "38" } else { "48" }, r, g, b);
            }
            Color::None => buf.push_str(if fg { "39m" } else { "49m" }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_default() {
        let color = Color::default();
        assert_eq!(color, Color::None);
    }
}
