use rustui::*;

#[test]
fn test_attr_to_ansi() {
    let attr = Attr::BOLD | Attr::UNDERLINE;
    assert_eq!(attr.to_ansi(), "\x1B[1;4m");

    let attr = Attr::empty();
    assert_eq!(attr.to_ansi(), "\x1B[0m");

    let attr = Attr::all();
    assert_eq!(attr.to_ansi(), "\x1B[0;1;2;3;4;5;6;7;8;9m");
}

#[test]
fn test_color_init() {
    let color = Color::new();
    assert_eq!(color, (-1, -1, -1));
}

#[test]
fn test_color_is_valid() {
    let color = Color::new();
    assert!(!color.is_valid());

    let color = (255, 255, 255);
    assert!(color.is_valid());

    let color = (-1, 128, 128);
    assert!(!color.is_valid());

    let color = (256, 128, 128);
    assert!(!color.is_valid());
}
