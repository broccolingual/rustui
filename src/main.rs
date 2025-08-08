use rcurses::*;
use std::{thread, time};

pub mod block;
pub mod core;
pub mod field;

use core::*;

const RENDERING_RATE: time::Duration = time::Duration::from_millis(16); // ms
const INPUT_CAPTURING_RATE: time::Duration = time::Duration::from_millis(10); // ms

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut win = Window::new()?;
    win.init()?;
    let fps_rx = win.start(RENDERING_RATE);
    let (mut key_listener, key_rx) = KeyListener::new(INPUT_CAPTURING_RATE);

    if win.width < 60 || win.height < 30 {
        key_listener.stop()?;
        win.end()?;
        return Err(Box::from(
            "Window size is too small. Minimum size is 60x30.",
        ));
    }

    let x_center = win.width / 2;
    let y_center: usize = win.height / 2;

    let mut fps: f64 = 0.0;
    let mut last_key = None;
    let mut info_frame = Framebuffer::new(32, 2);

    {
        let mut back_locked = win.get_lock();
        back_locked.set_str(
            x_center,
            y_center - 4,
            r" _____ _____ _____ ____  ___ ____ ",
            style![Attr::BOLD, Color::Fg24(96)],
            Align::Center,
        );
        back_locked.set_str(
            x_center,
            y_center - 3,
            r"|_   _| ____|_   _|  _ \|_ _/ ___| ",
            style![Attr::BOLD, Color::Fg24(96)],
            Align::Center,
        );
        back_locked.set_str(
            x_center,
            y_center - 2,
            r"  | | |  _|   | | | |_) || |\___ \ ",
            style![Attr::BOLD, Color::Fg24(96)],
            Align::Center,
        );
        back_locked.set_str(
            x_center,
            y_center - 1,
            r"  | | | |___  | | |  _ < | | ___) |",
            style![Attr::BOLD, Color::Fg24(96)],
            Align::Center,
        );
        back_locked.set_str(
            x_center,
            y_center - 0,
            r"  |_| |_____| |_| |_| \_\___|____/",
            style![Attr::BOLD, Color::Fg24(96)],
            Align::Center,
        );
        back_locked.set_border(style![Attr::NORMAL]);
    }

    thread::sleep(time::Duration::from_secs(2));

    let mut core = Core::new();

    let drop_counter_max = 60; // 1 = 1 frame, 60 = 1 second
    let mut drop_counter = 0;
    let mut is_gameover = false;

    loop {
        if let Ok(v) = fps_rx.try_recv() {
            fps = v;
        }

        info_frame.clear();
        info_frame.set_str(
            1,
            0,
            &format!("FPS: {:.2}", &fps),
            style![Attr::ITALIC, Attr::NORMAL, Color::Fg24(74)],
            Align::Left,
        );
        info_frame.set_str(
            1,
            1,
            &format!("Key: {:?}", last_key),
            style![Attr::ITALIC, Attr::NORMAL, Color::Fg24(64)],
            Align::Left,
        );
        info_frame.set_color(Color::Bg24(235));

        if let Ok(key) = key_rx.try_recv() {
            last_key = Some(key.clone());
            match key {
                Key::Char('q') => break,
                Key::Char('r') => {
                    core.hold();
                }
                Key::Char(' ') => {
                    core.rotate();
                }
                Key::ArrowUp => {}
                Key::ArrowDown => {
                    core.move_down();
                }
                Key::ArrowRight => {
                    core.move_right();
                }
                Key::ArrowLeft => {
                    core.move_left();
                }
                _ => (),
            }
        }

        if drop_counter >= drop_counter_max {
            drop_counter = 0;
            core.move_down();
        }

        if core.is_gameover() {
            is_gameover = true;
            break;
        }

        core.proc_before_draw();

        {
            let mut back_locked = win.get_lock();
            back_locked.clear();
            back_locked.set_border(style![Attr::NORMAL]);
            back_locked.combine(
                &core.field_frame,
                x_center - core.field_frame.width / 2,
                y_center - core.field_frame.height / 2,
            );
            back_locked.combine(
                &core.holding_block_frame,
                x_center - core.field_frame.width / 2 - core.holding_block_frame.width - 2,
                y_center - core.field_frame.height / 2 + 1,
            );
            back_locked.combine(
                &core.next_block_frame,
                x_center + core.field_frame.width / 2 + 2,
                y_center - core.field_frame.height / 2 + 1,
            );
            back_locked.combine(&info_frame, 2, 1);
        }

        core.proc_after_draw();

        drop_counter += 1;

        thread::sleep(time::Duration::from_millis(16));
    }

    if is_gameover {
        {
            let mut back_locked = win.get_lock();
            back_locked.clear();
            back_locked.set_border(style![Attr::NORMAL]);
            back_locked.set_str(
                x_center,
                y_center,
                "G A M E  O V E R",
                style![Attr::BOLD, Color::Fg24(196)],
                Align::Center,
            );
        }
        thread::sleep(time::Duration::from_secs(2));
    }

    key_listener.stop()?;
    win.end()?;
    Ok(())
}
