use rustui::*;
use std::{thread, time};

const RENDERING_RATE: time::Duration = time::Duration::from_millis(16);
const INPUT_CAPTURING_RATE: time::Duration = time::Duration::from_millis(10); // ms

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut win = Window::new(false)?;
    win.init()?; // Initialize the window (enable raw mode)
    win.start(RENDERING_RATE); // Start the rendering thread

    // Create a key listener
    let key_rx = KeyListener::new(INPUT_CAPTURING_RATE);

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

        // Render the frame
        {
            let mut canvas = win.get_canvas();
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
        }

        thread::sleep(time::Duration::from_millis(100)); // Sleep to prevent high CPU usage
    }
    Ok(())
}
