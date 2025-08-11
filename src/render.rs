use std::sync::TryLockError;
use std::sync::{
    mpsc::{self, Receiver, Sender},
    Arc, Mutex,
};
use std::thread;
use std::time;

use crate::Framebuffer;

/// Represents a render thread for rendering frames.
pub struct RenderThread {
    /// The thread handle for the rendering thread.
    pub handle: Option<thread::JoinHandle<()>>,
    /// The stop signal sender for the rendering thread.
    pub stop_signal: Option<Sender<()>>,
}

impl RenderThread {
    /// Create a new render thread that renders frames at a specified rate.
    ///
    /// * `front_fb` - The front framebuffer to render to.
    /// * `back_fb` - The back framebuffer to render from.
    /// * `rendering_rate` - The rate at which to render frames.
    ///
    /// Returns a receiver for the frame rate updates.
    pub fn new(
        front_fb: Arc<Mutex<Framebuffer>>,
        back_fb: Arc<Mutex<Framebuffer>>,
        rendering_rate: time::Duration,
    ) -> Receiver<f64> {
        let (fps_tx, fps_rx): (Sender<f64>, Receiver<f64>) = mpsc::channel();
        let (_, stop_rx): (Sender<()>, Receiver<()>) = mpsc::channel();

        let _ = thread::spawn(move || {
            let mut last_sec = time::Instant::now();
            let mut frame_count = 0;
            let mut last_frame_time = time::Instant::now();

            loop {
                if stop_rx.try_recv().is_ok() {
                    break; // Stop the loop if a stop signal is received
                }

                // Frame rate control
                let elapsed_since_frame = last_frame_time.elapsed();
                if elapsed_since_frame < rendering_rate {
                    thread::sleep(rendering_rate - elapsed_since_frame);
                }
                last_frame_time = time::Instant::now();

                // Rendering process
                match back_fb.try_lock() {
                    Ok(back_locked) => {
                        if let Ok(mut front_locked) = front_fb.try_lock() {
                            if front_locked.refresh(&back_locked).is_err() {
                                continue;
                            }
                            frame_count += 1;
                        }
                    }
                    Err(TryLockError::WouldBlock) => {
                        // Skip if the back buffer is locked
                        thread::sleep(time::Duration::from_millis(1));
                        continue;
                    }
                    Err(_) => {
                        break;
                    }
                }

                // FPS calculation and sending
                let elapsed = last_sec.elapsed();
                if elapsed >= time::Duration::from_secs(1) {
                    let fps = frame_count as f64 / elapsed.as_secs_f64();
                    if fps_tx.send(fps).is_err() {
                        break; // Stop the loop if the receiver is closed
                    }
                    frame_count = 0;
                    last_sec = time::Instant::now();
                }
            }
        });
        fps_rx
    }

    /// Stop the render thread
    ///
    /// Returns `Ok(())` if the thread was stopped successfully, or an error if it failed.
    pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(tx) = self.stop_signal.take() {
            tx.send(())?; // Send stop signal
        }
        if let Some(handle) = self.handle.take() {
            handle.join().map_err(|_| "Failed to join render thread")?; // Wait for the thread to finish
        }
        Ok(())
    }
}

impl Drop for RenderThread {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
