use rustui::*;
use std::{thread, time};

pub mod block;
pub mod core;
pub mod field;

use core::*;

const RENDERING_RATE: time::Duration = time::Duration::from_millis(16); // ms
const INPUT_CAPTURING_RATE: time::Duration = time::Duration::from_millis(10); // ms
const DROP_COUNTER_MAX: usize = 30; // 1 = 1 frame, 60 = 1 second
const MOVING_AFTER_DROP_COUNTER_MAX: usize = 30; // 30 frames after drop

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut win = Window::new(true)?;
    win.init()?;
    win.start(RENDERING_RATE);
    let key_rx = KeyListener::new(INPUT_CAPTURING_RATE);

    if win.width < 60 || win.height < 30 {
        return Err(Box::from(
            "Window size is too small. Minimum size is 60x30.",
        ));
    }

    let x_center = win.width / 2;
    let y_center = win.height / 2;

    let mut core = Core::new(DROP_COUNTER_MAX, MOVING_AFTER_DROP_COUNTER_MAX);

    loop {
        if core.is_gameover {
            break;
        }

        if let Ok(key) = key_rx.try_recv() {
            match key {
                Key::Char('q') => break,
                Key::Char('r') => {
                    core.hold();
                }
                Key::Char(' ') => {
                    core.quick_drop();
                }
                Key::ArrowUp => {
                    core.rotate();
                }
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

        core.proc_before_draw();

        win.draw(|canvas| {
            canvas.set_border(term::Attr::NORMAL, (255, 255, 255), Color::new());
            canvas.combine(
                &core.field_frame,
                x_center - core.field_frame.width / 2,
                y_center - core.field_frame.height / 2,
            );
            canvas.combine(
                &core.holding_block_frame,
                x_center - core.field_frame.width / 2 - core.holding_block_frame.width - 2,
                y_center - core.field_frame.height / 2 + 1,
            );
            canvas.combine(
                &core.next_block_frame,
                x_center + core.field_frame.width / 2 + 2,
                y_center - core.field_frame.height / 2 + 1,
            );
        });

        core.proc_after_draw();

        thread::sleep(time::Duration::from_millis(16));
    }

    if core.is_gameover {
        win.draw(|canvas| {
            canvas.set_border(term::Attr::BOLD, (255, 0, 0), (0, 0, 0));
            canvas.set_str(
                x_center,
                y_center,
                "G A M E  O V E R",
                term::Attr::BOLD,
                (255, 128, 128),
                (0, 0, 0),
                Align::Center,
            );
        });
        thread::sleep(time::Duration::from_secs(2));
    }
    Ok(())
}
