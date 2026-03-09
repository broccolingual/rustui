use std::sync::TryLockError;
use std::sync::{
    mpsc::{self, Receiver, Sender, SyncSender},
    Arc, Mutex,
};
use std::thread;
use std::time;

use crate::Framebuffer;

/// Represents a render thread for rendering frames.
pub struct RenderThread {
    /// The thread handle for the render thread.
    handle: Option<thread::JoinHandle<()>>,
    /// The stop signal sender for the render thread.
    stop_signal: Option<Sender<()>>,
    /// The frame rate receiver for the render thread.
    fps_rx: Receiver<f64>,
}

impl RenderThread {
    /// Create a new render thread that renders frames at a specified rate.
    ///
    /// * `front_fb` - The front framebuffer to render to.
    /// * `back_fb` - The back framebuffer to render from.
    /// * `rendering_rate` - The rate at which to render frames.
    ///
    /// Returns `RenderThread` instance.
    pub fn new(
        front_fb: Arc<Mutex<Framebuffer>>,
        back_fb: Arc<Mutex<Framebuffer>>,
        rendering_rate: time::Duration,
    ) -> Self {
        let (fps_tx, fps_rx): (SyncSender<f64>, Receiver<f64>) = mpsc::sync_channel(1);
        let (stop_tx, stop_rx): (Sender<()>, Receiver<()>) = mpsc::channel();

        let handle = thread::spawn(move || {
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

                // Rendering process
                match back_fb.try_lock() {
                    Ok(back_locked) => {
                        if let Ok(mut front_locked) = front_fb.try_lock() {
                            if front_locked.refresh(&back_locked).is_err() {
                                continue;
                            }
                            // Update frame time only after a successful render so that
                            // a failed lock attempt does not consume the frame budget.
                            last_frame_time = time::Instant::now();
                            frame_count += 1;
                        }
                    }
                    Err(TryLockError::WouldBlock) => {
                        // back_fb is locked by draw(); retry immediately without
                        // resetting the frame timer so the next iteration does not
                        // sleep a full rendering_rate before retrying.
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
                    match fps_tx.try_send(fps) {
                        Ok(_) => {}
                        Err(mpsc::TrySendError::Full(_)) => {} // If the channel is full, we can drop the event
                        Err(mpsc::TrySendError::Disconnected(_)) => break, // Stop the loop if the receiver is dropped
                    }
                    frame_count = 0;
                    last_sec = time::Instant::now();
                }
            }
        });
        Self {
            handle: Some(handle),
            stop_signal: Some(stop_tx),
            fps_rx,
        }
    }

    /// Try to receive the current FPS
    ///
    /// Returns the current FPS if available, or an error if not.
    pub fn try_recv_fps(&self) -> Result<f64, mpsc::TryRecvError> {
        self.fps_rx.try_recv()
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
