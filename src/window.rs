use std::io;
use std::os::unix::io::AsRawFd;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time;

use crate::{Align, Attr, Cmd, Color, ColorExt, Framebuffer, RenderThread, Terminal};

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
    fps: f64,
    show_fps: bool,
}

impl Window {
    /// Create a new window with the specified dimensions and an option to show FPS.
    ///
    /// * `show_fps` - Whether to display the FPS counter.
    ///
    /// Returns a `Window` instance if successful, or an error if it failed.
    pub fn new(show_fps: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let (width, height) = Terminal::get_size()?;
        let front_fb = Arc::new(Mutex::new(Framebuffer::new(width, height)));
        let back_fb = Arc::new(Mutex::new(Framebuffer::new(width, height)));
        Ok(Self {
            width,
            height,
            front_fb,
            back_fb,
            terminal: None,
            render_thread: None,
            fps: 0.0,
            show_fps,
        })
    }

    /// Change to raw mode
    #[deprecated(since = "0.1.11", note = "Use `initialize()` method instead")]
    pub fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let fd = io::stdin().as_raw_fd();
        let terminal = Terminal::enable_raw_mode(fd)?;
        terminal.set_nonblocking()?;

        Terminal::exec(Cmd::EnableAlternativeScreen)?;
        Terminal::exec(Cmd::HideCursor)?;
        Terminal::exec(Cmd::EnableMouseReporting)?;
        Terminal::exec(Cmd::EnableSgrCoords)?;

        self.terminal = Some(terminal);
        Ok(())
    }

    /// Start the rendering thread
    #[deprecated(since = "0.1.11", note = "Use `initialize()` method instead")]
    pub fn start(&mut self, rate: time::Duration) {
        let render_thread =
            RenderThread::new(Arc::clone(&self.front_fb), Arc::clone(&self.back_fb), rate);
        self.render_thread = Some(render_thread);
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
        let fd = io::stdin().as_raw_fd();
        let terminal = Terminal::enable_raw_mode(fd)?;
        terminal.set_nonblocking()?;
        Terminal::exec(Cmd::EnableAlternativeScreen)?;
        Terminal::exec(Cmd::HideCursor)?;
        Terminal::exec(Cmd::EnableMouseReporting)?;
        Terminal::exec(Cmd::EnableSgrCoords)?;
        self.terminal = Some(terminal);
        self.render_thread = Some(RenderThread::new(
            Arc::clone(&self.front_fb),
            Arc::clone(&self.back_fb),
            rendering_rate,
        ));
        Ok(())
    }

    /// Get a mutable reference to the buffer's Mutex
    #[deprecated(
        since = "0.1.11",
        note = "Use `draw()` method instead for better encapsulation"
    )]
    pub fn get_canvas(&mut self) -> MutexGuard<'_, Framebuffer> {
        let fps = self.get_fps();
        let mut canvas = self.back_fb.lock().unwrap();
        canvas.clear();
        if self.show_fps {
            canvas.set_str(
                2,
                1,
                &format!("FPS: {fps:.2}"),
                Attr::NORMAL | Attr::BOLD,
                (128, 255, 128),
                Color::new(),
                Align::Left,
            );
        }
        canvas
    }

    /// Draw the contents of the framebuffer
    ///
    /// * `f` - A closure that takes a mutable reference to the framebuffer.
    pub fn draw(&mut self, f: impl FnOnce(&mut Framebuffer)) {
        let fps = self.get_fps();
        let mut lock = self.back_fb.lock().unwrap();
        lock.clear();
        if self.show_fps {
            lock.set_str(
                2,
                1,
                &format!("FPS: {fps:.2}"),
                Attr::NORMAL | Attr::BOLD,
                (128, 255, 128),
                Color::new(),
                Align::Left,
            );
        }
        f(&mut lock);
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
            self.fps = fps;
        }
        self.fps
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        if let Err(e) = self.end() {
            eprintln!("Error restoring terminal state: {e}");
        }
    }
}
