use rustui::*;
use std::{thread, time};

const RENDERING_RATE: time::Duration = time::Duration::from_millis(16);
const INPUT_CAPTURING_RATE: time::Duration = time::Duration::from_millis(10); // ms

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut win = Window::new(false)?;
    win.initialize(RENDERING_RATE)?; // Initialize the window and start the rendering thread
    let key_rx = KeyListener::new(INPUT_CAPTURING_RATE); // Create a key listener

    let x_center = win.width / 2;
    let y_center = win.height / 2;

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
            canvas.set_border(term::Attr::NORMAL, (255, 255, 255), Color::new());
            canvas.set_str(
                x_center,
                y_center,
                "Hello, world! (Press 'q' to quit)",
                term::Attr::NORMAL,
                (128, 255, 128),
                Color::new(),
                Align::Center,
            );
        });

        thread::sleep(time::Duration::from_millis(100)); // Sleep to prevent high CPU usage
    }
    Ok(())
}
