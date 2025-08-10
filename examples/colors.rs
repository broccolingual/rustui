use rustui::*;
use std::{thread, time};

const RENDERING_RATE: time::Duration = time::Duration::from_millis(16);
const INPUT_CAPTURING_RATE: time::Duration = time::Duration::from_millis(10); // ms

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut win = Window::new(false)?;
    win.initialize(RENDERING_RATE)?; // Initialize the window and start the rendering thread
    let key_rx = KeyListener::new(INPUT_CAPTURING_RATE); // Create a key listener

    loop {
        // Check for key presses
        if let Ok(key) = key_rx.try_recv() {
            match key {
                Key::Char('q') => break,
                _ => (),
            }
        }

        // Draw the frame
        win.draw(|canvas| {
            for r in 0..8 {
                for g in 0..8 {
                    for b in 0..8 {
                        let color = (r * 32 as i32, g * 32 as i32, b * 32 as i32);
                        canvas.set_str(
                            (r * 2 + b * 18) as usize,
                            g as usize,
                            "  ",
                            term::Attr::NORMAL,
                            Color::new(),
                            color,
                            Align::Left,
                        );
                    }
                }
            }
            canvas.set_str(
                0,
                10,
                "Press 'q' to quit",
                term::Attr::BOLD,
                Color::new(),
                Color::new(),
                Align::Left,
            );
        });

        thread::sleep(time::Duration::from_millis(100)); // Sleep to prevent high CPU usage
    }
    Ok(())
}
