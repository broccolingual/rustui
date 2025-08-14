use crate::csi;
use bitflags::bitflags;

bitflags! {
    /// Represents terminal attributes using bitflags.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Attr: u16 {
        const NORMAL = 1; // 0
        const BOLD = 2; // 1
        const THIN = 4; // 2
        const ITALIC = 8; // 3
        const UNDERLINE = 16; // 4
        const BLINK = 32; // 5
        const FASTBLINK = 64; // 6
        const INVERT = 128; // 7
        const HIDDEN = 256; // 8
        const REMOVE = 512; // 9
        const PRIMARY = 1024; // 10
    }
}

impl Default for Attr {
    fn default() -> Self {
        Attr::NORMAL
    }
}

impl Attr {
    /// Convert attributes to ANSI escape codes
    ///
    /// Returns a string containing the ANSI escape codes for the active attributes.
    pub fn to_ansi(&self) -> String {
        if self.is_empty() {
            return csi!("0m");
        }

        let attr_mappings = [
            (Attr::NORMAL, "0"),
            (Attr::BOLD, "1"),
            (Attr::THIN, "2"),
            (Attr::ITALIC, "3"),
            (Attr::UNDERLINE, "4"),
            (Attr::BLINK, "5"),
            (Attr::FASTBLINK, "6"),
            (Attr::INVERT, "7"),
            (Attr::HIDDEN, "8"),
            (Attr::REMOVE, "9"),
            (Attr::PRIMARY, "10"),
        ];

        let mut buf = String::with_capacity(24);
        buf.push_str("\x1B[");

        let mut first = true;
        for (flag, code) in attr_mappings.iter() {
            if self.contains(*flag) {
                if !first {
                    buf.push(';');
                }
                buf.push_str(code);
                first = false;
            }
        }

        buf.push('m');
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attr_default() {
        let attr = Attr::default();
        assert_eq!(attr, Attr::NORMAL);
    }

    #[test]
    fn test_attr_to_ansi() {
        let attr = Attr::BOLD | Attr::UNDERLINE;
        assert_eq!(attr.to_ansi(), "\x1B[1;4m");

        let attr = Attr::empty();
        assert_eq!(attr.to_ansi(), "\x1B[0m");

        let attr = Attr::all();
        assert_eq!(attr.to_ansi(), "\x1B[0;1;2;3;4;5;6;7;8;9;10m");
    }
}

