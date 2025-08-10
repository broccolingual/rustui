use std::io;
use std::os::unix::io::AsRawFd;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time;

use crate::*;

pub struct Window {
    pub width: usize,
    pub height: usize,
    front_fb: Arc<Mutex<Framebuffer>>,
    back_fb: Arc<Mutex<Framebuffer>>,
    terminal: Option<Terminal>,
    fps_rx: Receiver<f64>,
    fps: f64,
    show_fps: bool,
}

impl Window {
    pub fn new(show_fps: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let (width, height) = Terminal::get_size()?;
        let front_fb = Arc::new(Mutex::new(Framebuffer::new(width, height)));
        let back_fb = Arc::new(Mutex::new(Framebuffer::new(width, height)));
        let (_, fps_rx) = std::sync::mpsc::channel();
        Ok(Self {
            width,
            height,
            front_fb,
            back_fb,
            terminal: None,
            fps_rx,
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
        let fps_rx = RenderThread::new(Arc::clone(&self.front_fb), Arc::clone(&self.back_fb), rate);
        self.fps_rx = fps_rx;
    }

    /// Initialize the window and start the rendering thread
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
        self.fps_rx = RenderThread::new(
            Arc::clone(&self.front_fb),
            Arc::clone(&self.back_fb),
            rendering_rate,
        );
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
    pub fn end(&mut self) -> io::Result<()> {
        Terminal::exec(Cmd::DisableSgrCoords)?;
        Terminal::exec(Cmd::DisableMouseReporting)?;
        Terminal::exec(Cmd::ShowCursor)?;
        Terminal::exec(Cmd::DisableAlternativeScreen)?;
        Ok(())
    }

    /// Get the current FPS
    fn get_fps(&mut self) -> f64 {
        if let Ok(fps) = self.fps_rx.try_recv() {
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
