use rustui::*;
use std::{thread, time};

const RENDERING_RATE: time::Duration = time::Duration::from_millis(16); // ms
const INPUT_CAPTURING_RATE: time::Duration = time::Duration::from_millis(10); // ms

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut win = Window::new(false)?;
    win.initialize(RENDERING_RATE)?; // Initialize the window and start the rendering thread
    let key_rx = KeyListener::new(INPUT_CAPTURING_RATE); // Create a key listener

    let x_center = win.width / 2;
    let y_center = win.height / 2;

    let mut key_last_pressed = None;

    loop {
        // Check for key presses
        if let Ok(key) = key_rx.try_recv() {
            match key {
                Key::Char('q') => break,
                _ => (),
            }
            key_last_pressed = Some(key);
        }

        // Draw the frame
        win.draw(|canvas| {
            canvas.set_border(Attr::NORMAL, (255, 255, 255), Color::new()); // Set border
            canvas.set_str(
                x_center,
                y_center,
                &format!(
                    "You pressed: {:?} (Press 'q' to quit)",
                    key_last_pressed.unwrap_or(Key::Unknown)
                ),
                Attr::NORMAL,    // Set text decoration
                (255, 255, 255), // Set text color
                Color::new(),    // Set background color
                Align::Center,   // Set text alignment to center
            );
        });

        thread::sleep(time::Duration::from_millis(100)); // Sleep to prevent high CPU usage
    }
    Ok(())
}
