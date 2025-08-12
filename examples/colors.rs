use rustui::*;
use std::{thread, time};

const RENDERING_RATE: time::Duration = time::Duration::from_millis(16);
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
                "COLORS",
                Align::Right,
                Attr::NORMAL,
                Color::White,
                Color::default(),
            );

            // Draw color circle
            let outer_radius: i32 = 12;
            let inner_radius: i32 = 6;
            for hue in 0..360 {
                for radius in inner_radius..outer_radius {
                    let x = (radius as f32 * (hue as f32).to_radians().cos()) as i32;
                    let y = (radius as f32 * (hue as f32).to_radians().sin()) as i32;
                    let sat = 255 * (radius - inner_radius - outer_radius)
                        / (outer_radius - inner_radius);
                    let color = Color::HSV((hue as f32 / 360.0 * 255.0) as u8, sat as u8, 255);
                    canvas.set_str(
                        ((canvas.width / 2) as i32 + x * 2) as usize,
                        ((canvas.height / 2) as i32 + y) as usize,
                        "  ",
                        Attr::NORMAL,
                        Color::default(),
                        color,
                        Align::Left,
                    );
                }
            }
            canvas.set_str(
                canvas.width / 2 + 1,
                canvas.height / 2,
                "Color Circle",
                Attr::BOLD,
                Color::White,
                Color::default(),
                Align::Center,
            );
            canvas.set_str(
                3,
                2,
                "Press 'q' to quit",
                Attr::BOLD,
                Color::White,
                Color::default(),
                Align::Left,
            );
        })?;

        thread::sleep(time::Duration::from_millis(100)); // Sleep to prevent high CPU usage
    }
    Ok(())
}
