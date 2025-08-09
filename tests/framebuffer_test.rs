use rustui::*;

#[test]
fn test_cell_init() {
    let cell = Cell::new();
    assert_eq!(cell.ch, ' ');
    assert_eq!(cell.attrs, Attr::NORMAL);
    assert_eq!(cell.bg, Color::new());
    assert_eq!(cell.fg, Color::new());
}

#[test]
fn test_fb_init() {
    let fb = Framebuffer::new(4, 2);
    assert_eq!(fb.width, 4);
    assert_eq!(fb.height, 2);
    assert_eq!(fb.buffer.len(), 8);
}

#[test]
fn test_fb_check_range() {
    let fb = Framebuffer::new(4, 2);
    assert!(fb.check_range(0, 0));
    assert!(fb.check_range(3, 1));
    assert!(!fb.check_range(4, 0));
    assert!(!fb.check_range(0, 2));
}

#[test]
fn test_fb_clear() {
    let mut fb = Framebuffer::new(4, 2);
    fb.set_char(0, 0, 'X', Attr::NORMAL, Color::new(), Color::new());
    fb.clear();
    for cell in fb.buffer {
        assert_eq!(cell.ch, ' ');
        assert_eq!(cell.attrs, Attr::NORMAL);
        assert_eq!(cell.bg, Color::new());
        assert_eq!(cell.fg, Color::new());
    }
}

#[test]
fn test_fb_set_char() {
    let mut fb = Framebuffer::new(4, 2);
    fb.set_char(0, 0, 'X', Attr::NORMAL, Color::new(), Color::new());
    assert_eq!(fb.buffer[0].ch, 'X');
    assert_eq!(fb.buffer[0].attrs, Attr::NORMAL);
    assert_eq!(fb.buffer[0].bg, Color::new());
    assert_eq!(fb.buffer[0].fg, Color::new());
}

#[test]
fn test_fb_set_str() {
    let mut fb = Framebuffer::new(4, 2);
    fb.set_str(
        0,
        0,
        "XY",
        Attr::NORMAL,
        Color::new(),
        Color::new(),
        Align::Left,
    );
    assert_eq!(fb.buffer[0].ch, 'X');
    assert_eq!(fb.buffer[1].ch, 'Y');
}

#[test]
fn test_fb_set_border() {
    let mut fb = Framebuffer::new(4, 3);
    fb.set_border(Attr::NORMAL, (255, 255, 255), Color::new());
    assert_eq!(fb.buffer[0].ch, '┌');
    assert_eq!(fb.buffer[1].ch, '─');
    assert_eq!(fb.buffer[2].ch, '─');
    assert_eq!(fb.buffer[3].ch, '┐');
    assert_eq!(fb.buffer[4].ch, '│');
    assert_eq!(fb.buffer[5].ch, ' ');
    assert_eq!(fb.buffer[6].ch, ' ');
    assert_eq!(fb.buffer[7].ch, '│');
    assert_eq!(fb.buffer[8].ch, '└');
    assert_eq!(fb.buffer[9].ch, '─');
    assert_eq!(fb.buffer[10].ch, '─');
    assert_eq!(fb.buffer[11].ch, '┘');
}

#[test]
fn test_fb_set_vline() {
    let mut fb = Framebuffer::new(4, 3);
    fb.set_vline(1, 0, 3, Attr::NORMAL, Color::new(), Color::new());
    assert_eq!(fb.buffer[1].ch, '│');
    assert_eq!(fb.buffer[5].ch, '│');
    assert_eq!(fb.buffer[9].ch, '│');
}

#[test]
fn test_fb_set_hline() {
    let mut fb = Framebuffer::new(4, 3);
    fb.set_hline(1, 0, 4, Attr::NORMAL, Color::new(), Color::new());
    assert_eq!(fb.buffer[4].ch, '─');
    assert_eq!(fb.buffer[5].ch, '─');
    assert_eq!(fb.buffer[6].ch, '─');
    assert_eq!(fb.buffer[7].ch, '─');
}

#[test]
fn test_fb_combine() {
    let mut fb1 = Framebuffer::new(3, 3);
    fb1.set_border(Attr::NORMAL, Color::new(), Color::new());
    let mut fb2 = Framebuffer::new(1, 1);
    fb2.set_str(
        0,
        0,
        "A",
        Attr::BOLD,
        Color::new(),
        Color::new(),
        Align::Left,
    );
    fb1.combine(&fb2, 1, 1);
    assert_eq!(fb1.buffer[0].ch, '┌');
    assert_eq!(fb1.buffer[1].ch, '─');
    assert_eq!(fb1.buffer[2].ch, '┐');
    assert_eq!(fb1.buffer[3].ch, '│');
    assert_eq!(fb1.buffer[4].ch, 'A');
    assert_eq!(fb1.buffer[4].attrs, Attr::BOLD);
    assert_eq!(fb1.buffer[5].ch, '│');
    assert_eq!(fb1.buffer[6].ch, '└');
    assert_eq!(fb1.buffer[7].ch, '─');
    assert_eq!(fb1.buffer[8].ch, '┘');
}
