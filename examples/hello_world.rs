use rustui::*;
use std::{thread, time};

const RENDERING_RATE: time::Duration = time::Duration::from_millis(16); // ms
const INPUT_CAPTURING_RATE: time::Duration = time::Duration::from_millis(10); // ms

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut win = Window::new(false)?;
    win.initialize(RENDERING_RATE)?; // Initialize the window and start the rendering thread
    let input_rx = InputListener::new(INPUT_CAPTURING_RATE); // Create an input listener

    loop {
        // Check for key presses
        if let Ok(InputEvent::Key(Key::Char('q'))) = input_rx.try_recv() {
            break; // Exit the loop if 'q' is pressed
        }

        // Draw the frame
        win.draw(|canvas| {
            canvas.set_named_border(
                "HELLO WORLD",
                Align::Right,
                Attr::NORMAL,
                (255, 255, 255),
                Color::new(),
            ); // Set a named border for the canvas
            canvas.set_str(
                canvas.width / 2, // Center the text horizontally
                canvas.height / 2,
                "Hello, world! (Press 'q' to quit)",
                Attr::NORMAL,    // Set text decoration
                (128, 255, 128), // Set text color
                (64, 64, 64),    // Set background color
                Align::Center,   // Set text alignment to center
            );
        })?;

        thread::sleep(time::Duration::from_millis(100)); // Sleep to prevent high CPU usage
    }
    Ok(())
}
