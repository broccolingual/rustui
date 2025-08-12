use rustui::*;
use std::{thread, time};

const RENDERING_RATE: time::Duration = time::Duration::from_millis(16); // ms
const INPUT_CAPTURING_RATE: time::Duration = time::Duration::from_millis(10); // ms

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut win = Window::new(false)?;
    win.initialize(RENDERING_RATE)?; // Initialize the window and start the rendering thread
    let input_rx = InputListener::new(INPUT_CAPTURING_RATE); // Create an input listener
    let mut key_last_pressed = None;

    loop {
        // Check for key presses
        if let Ok(event) = input_rx.try_recv() {
            match event {
                InputEvent::Key(Key::Char('q')) => break,
                _ => (),
            }
            key_last_pressed = Some(event);
        }

        // Draw the frame
        win.draw(|canvas| {
            canvas.set_border(Attr::NORMAL, (255, 255, 255), Color::new()); // Set border

            let display_text = match key_last_pressed {
                Some(InputEvent::Key(key)) => format!("Key: {:?}", key),
                Some(InputEvent::Mouse(mouse)) => match mouse {
                    MouseEvent::Press { button, x, y } => {
                        format!("Mouse Press: {:?} at ({},{})", button, x, y)
                    }
                    MouseEvent::Release { button, x, y } => {
                        format!("Mouse Release: {:?} at ({},{})", button, x, y)
                    }
                    MouseEvent::Move { x, y } => {
                        format!("Mouse Move: at ({},{})", x, y)
                    }
                },
                None => "No input yet".to_string(),
            };

            let full_text = format!("{} (Press 'q' to quit)", display_text);

            canvas.set_str(
                canvas.width / 2,
                canvas.height / 2,
                &full_text,
                Attr::NORMAL,
                (255, 255, 255),
                Color::new(),
                Align::Center,
            );
        })?;

        thread::sleep(time::Duration::from_millis(100)); // Sleep to prevent high CPU usage
    }
    Ok(())
}
