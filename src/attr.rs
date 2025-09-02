use crate::csi;
use bitflags::bitflags;

bitflags! {
    /// Represents terminal attributes using bitflags.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Attr: u16 {
        const NORMAL = 0b0000_0000_0000_0001; // deprecated
        const Normal = 0b0000_0000_0000_0001;
        const BOLD = 0b0000_0000_0000_0010; // deprecated
        const Bold = 0b0000_0000_0000_0010;
        const THIN = 0b0000_0000_0000_0100; // deprecated
        const Thin = 0b0000_0000_0000_0100;
        const ITALIC = 0b0000_0000_0000_1000; // deprecated
        const Italic = 0b0000_0000_0000_1000;
        const UNDERLINE = 0b0000_0000_0001_0000; // deprecated
        const Underline = 0b0000_0000_0001_0000;
        const BLINK = 0b0000_0000_0010_0000; // deprecated
        const Blink = 0b0000_0000_0010_0000;
        const FASTBLINK = 0b0000_0000_0100_0000; // deprecated
        const FastBlink = 0b0000_0000_0100_0000;
        const INVERT = 0b0000_0000_1000_0000; // deprecated
        const Invert = 0b0000_0000_1000_0000;
        const HIDDEN = 0b0000_0001_0000_0000; // deprecated
        const Hidden = 0b0000_0001_0000_0000;
        const REMOVE = 0b0000_0010_0000_0000; // deprecated
        const Remove = 0b0000_0010_0000_0000;
        const PRIMARY = 0b0000_0100_0000_0000; // deprecated
        const Primary = 0b0000_0100_0000_0000;
    }
}

impl Default for Attr {
    fn default() -> Self {
        Attr::Normal
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
            (Attr::Normal, "0"),
            (Attr::Bold, "1"),
            (Attr::Thin, "2"),
            (Attr::Italic, "3"),
            (Attr::Underline, "4"),
            (Attr::Blink, "5"),
            (Attr::FastBlink, "6"),
            (Attr::Invert, "7"),
            (Attr::Hidden, "8"),
            (Attr::Remove, "9"),
            (Attr::Primary, "10"),
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
