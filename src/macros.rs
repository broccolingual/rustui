/// Create a CSI (Control Sequence Introducer) escape sequence
#[macro_export]
#[doc(hidden)]
macro_rules! csi {
    ($x:expr) => {
        String::from("\x1B[") + $x
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_csi_macro() {
        assert_eq!(csi!("?25h"), "\x1B[?25h");
    }
}
