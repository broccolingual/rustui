use std::io;
use std::sync::mpsc::{self, Receiver, Sender, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{self, Duration};

use crate::{Align, Attr, Cmd, Color, Framebuffer, RenderThread, Terminal};

const WINDOW_SIZE_CHANGE_DETECTION_RATE: time::Duration = time::Duration::from_millis(500); // ms

/// Represents a window with a framebuffer for rendering.
pub struct Window {
    /// The width of the window.
    pub width: usize,
    /// The height of the window.
    pub height: usize,
    front_fb: Arc<Mutex<Framebuffer>>,
    back_fb: Arc<Mutex<Framebuffer>>,
    terminal: Option<Terminal>,
    render_thread: Option<RenderThread>,
    window_size_listener: Option<WindowSizeListener>,
    actual_fps: f64,
    rendering_rate: Duration,
    show_fps: bool,
}

impl Window {
    /// Create a new window with the specified dimensions and an option to show FPS.
    ///
    /// * `show_fps` - Whether to display the FPS counter.
    ///
    /// Returns a `Window` instance if successful, or an error if it failed.
    pub fn new(show_fps: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let terminal = Terminal::new();
        let (width, height) = terminal.get_size()?;
        let front_fb = Arc::new(Mutex::new(Framebuffer::new(width, height)));
        let back_fb = Arc::new(Mutex::new(Framebuffer::new(width, height)));
        Ok(Self {
            width,
            height,
            front_fb,
            back_fb,
            terminal: Some(terminal),
            render_thread: None,
            window_size_listener: None,
            actual_fps: 0.0,
            rendering_rate: Duration::from_millis(16),
            show_fps,
        })
    }

    /// Initialize the window and start the rendering thread
    ///
    /// * `rendering_rate` - The rate at which to render frames.
    ///
    /// Returns `Ok(())` if the initialization was successful, or an error if it failed.
    pub fn initialize(
        &mut self,
        rendering_rate: time::Duration,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(terminal) = &mut self.terminal {
            terminal.enable_raw_mode()?;
            terminal.set_nonblocking()?;
            self.window_size_listener = Some(WindowSizeListener::new(
                self.terminal.take().unwrap(),
                WINDOW_SIZE_CHANGE_DETECTION_RATE,
            ));
        }

        Terminal::exec(Cmd::EnableAlternativeScreen)?;
        Terminal::exec(Cmd::HideCursor)?;
        Terminal::exec(Cmd::EnableMouseReporting)?;
        Terminal::exec(Cmd::EnableSgrCoords)?;

        self.rendering_rate = rendering_rate;
        self.render_thread = Some(RenderThread::new(
            Arc::clone(&self.front_fb),
            Arc::clone(&self.back_fb),
            rendering_rate,
        ));
        Ok(())
    }

    /// Draw the contents of the framebuffer
    ///
    /// * `f` - A closure that takes a mutable reference to the framebuffer.
    ///
    /// Returns `Ok(())` if the drawing was successful, or an error if it failed.
    pub fn draw(
        &mut self,
        f: impl FnOnce(&mut Framebuffer),
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.check_window_size_changes()?;
        let fps = self.get_fps();
        let mut lock = self.back_fb.lock().unwrap();
        lock.clear();
        if self.show_fps {
            lock.set_str(
                2,
                1,
                &format!("FPS: {fps:.2}"),
                Attr::BOLD,
                Color::Green,
                Color::default(),
                Align::Left,
            );
        }
        f(&mut lock);
        Ok(())
    }

    /// Resize the window
    ///
    /// * `width` - The new width of the window.
    /// * `height` - The new height of the window.
    ///
    /// Returns `Ok(())` if the resizing was successful, or an error if it failed.
    fn resize(&mut self, width: usize, height: usize) -> Result<(), Box<dyn std::error::Error>> {
        Terminal::exec(Cmd::ClearScreen)?;
        self.width = width;
        self.height = height;
        if let Some(render_thread) = &mut self.render_thread {
            render_thread.stop()?;
        }
        self.front_fb = Arc::new(Mutex::new(Framebuffer::new(width, height)));
        self.back_fb = Arc::new(Mutex::new(Framebuffer::new(width, height)));
        self.render_thread = Some(RenderThread::new(
            Arc::clone(&self.front_fb),
            Arc::clone(&self.back_fb),
            self.rendering_rate,
        ));
        Ok(())
    }

    /// Check for window size changes
    ///
    /// Returns `Ok(())` if the check was successful, or an error if it failed.
    fn check_window_size_changes(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(listener) = &self.window_size_listener {
            if let Ok((width, height)) = listener.try_recv() {
                self.resize(width, height)?;
            }
        }
        Ok(())
    }

    /// Restore the terminal
    ///
    /// Returns `Ok(())` if the terminal was restored successfully, or an error if it failed.
    pub fn end(&mut self) -> io::Result<()> {
        Terminal::exec(Cmd::DisableSgrCoords)?;
        Terminal::exec(Cmd::DisableMouseReporting)?;
        Terminal::exec(Cmd::ShowCursor)?;
        Terminal::exec(Cmd::DisableAlternativeScreen)?;
        Ok(())
    }

    /// Get the current FPS
    ///
    /// Returns the current frames per second.
    fn get_fps(&mut self) -> f64 {
        if let Ok(fps) = self.render_thread.as_ref().unwrap().try_recv_fps() {
            self.actual_fps = fps;
        }
        self.actual_fps
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        let _ = self.end();
    }
}

/// A listener for window size changes
pub struct WindowSizeListener {
    /// The thread handle for the listener
    handle: Option<thread::JoinHandle<()>>,
    /// The stop signal for the listener
    stop_signal: Option<Sender<()>>,
    /// The receiver for size change events
    size_rx: Receiver<(usize, usize)>,
}

impl WindowSizeListener {
    /// Create a new window size listener
    ///
    /// * `rate`: The rate at which to check for window size changes
    ///
    /// Returns `WindowSizeListener` instance.
    pub fn new(terminal: Terminal, rate: Duration) -> Self {
        #[allow(clippy::type_complexity)]
        let (size_tx, size_rx): (SyncSender<(usize, usize)>, Receiver<(usize, usize)>) =
            mpsc::sync_channel(1);
        let (stop_tx, stop_rx): (Sender<()>, Receiver<()>) = mpsc::channel();

        let handle = thread::spawn(move || {
            let mut prev_window_size: Option<(usize, usize)> = None; // Previous window size
            loop {
                if stop_rx.try_recv().is_ok() {
                    break; // Stop the loop if a stop signal is received
                }

                if let Ok((width, height)) = terminal.get_size() {
                    // Check if the window size has changed
                    if prev_window_size != Some((width, height)) {
                        match size_tx.try_send((width, height)) {
                            Ok(_) => {}
                            Err(mpsc::TrySendError::Full(_)) => {} // If the channel is full, we can drop the event
                            Err(mpsc::TrySendError::Disconnected(_)) => break, // Stop the loop if the receiver is dropped
                        }
                        prev_window_size = Some((width, height));
                    }
                }

                thread::sleep(rate);
            }
        });

        Self {
            handle: Some(handle),
            stop_signal: Some(stop_tx),
            size_rx,
        }
    }

    /// Try to receive a window size change event
    ///
    /// Returns the new window size if available, or an error if not.
    pub fn try_recv(&self) -> Result<(usize, usize), mpsc::TryRecvError> {
        self.size_rx.try_recv()
    }

    /// Stop the window size listener
    ///
    /// Returns `Ok(())` if the listener was stopped successfully, or an error if it failed.
    pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(tx) = self.stop_signal.take() {
            tx.send(())?; // Send stop signal
        }
        if let Some(handle) = self.handle.take() {
            handle
                .join()
                .map_err(|_| "Failed to join window size listener thread")?; // Wait for the thread to finish
        }
        Ok(())
    }
}

impl Drop for WindowSizeListener {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
