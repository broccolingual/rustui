use rustui::*;
use std::{thread, time};

const RENDERING_RATE: time::Duration = time::Duration::from_millis(16); // ms
const INPUT_CAPTURING_RATE: time::Duration = time::Duration::from_millis(10); // ms

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut win = Window::new(false)?;
    win.initialize(RENDERING_RATE)?; // Initialize the window and start the rendering thread
    let input_rx = InputListener::new(INPUT_CAPTURING_RATE); // Create an input listener

    let x_center = win.width / 2;
    let y_center = win.height / 2;

    loop {
        // Check for key presses
        if let Ok(event) = input_rx.try_recv() {
            match event {
                InputEvent::Key(Key::Char('q')) => break,
                _ => (),
            }
        }

        // Draw the frame
        win.draw(|canvas| {
            canvas.set_border(Attr::NORMAL, (255, 255, 255), Color::new()); // Set border
            canvas.set_str(
                x_center,
                y_center - 2,
                r"  ____  _   _ ____ _____ _   _ ___",
                Attr::BOLD,
                (255, 128, 128),
                Color::new(),
                Align::Center,
            );
            canvas.set_str(
                x_center,
                y_center - 1,
                r" |  _ \| | | / ___|_   _| | | |_ _|",
                Attr::BOLD,
                (255, 128, 128),
                Color::new(),
                Align::Center,
            );
            canvas.set_str(
                x_center,
                y_center,
                r"| |_) | | | \___ \ | | | | | || |",
                Attr::BOLD,
                (255, 128, 128),
                Color::new(),
                Align::Center,
            );
            canvas.set_str(
                x_center,
                y_center + 1,
                r"|  _ <| |_| |___) || | | |_| || |",
                Attr::BOLD,
                (255, 128, 128),
                Color::new(),
                Align::Center,
            );
            canvas.set_str(
                x_center,
                y_center + 2,
                r" |_| \_\\___/|____/ |_|  \___/|___|",
                Attr::BOLD,
                (255, 128, 128),
                Color::new(),
                Align::Center,
            );
            canvas.set_str(
                x_center + 1,
                y_center + 3,
                "The simplest terminal UI library",
                Attr::NORMAL,
                (255, 255, 255),
                Color::new(),
                Align::Center,
            );
        });

        thread::sleep(time::Duration::from_millis(100)); // Sleep to prevent high CPU usage
    }
    Ok(())
}
