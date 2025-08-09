use std::io;
use std::os::unix::io::AsRawFd;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time;

use crate::framebuffer;
use crate::render;
use crate::term;

pub struct Window {
    pub width: usize,
    pub height: usize,
    front_fb: Arc<Mutex<framebuffer::Framebuffer>>,
    back_fb: Arc<Mutex<framebuffer::Framebuffer>>,
    terminal: Option<term::Terminal>,
    render_thread: Option<render::RenderThread>,
    fps_rx: Receiver<f64>,
    fps: f64,
    debug: bool,
}

impl Window {
    /// コンストラクタ
    pub fn new(debug: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let (width, height) = term::Terminal::get_size()?;
        let front_fb = Arc::new(Mutex::new(framebuffer::Framebuffer::new(width, height)));
        let back_fb = Arc::new(Mutex::new(framebuffer::Framebuffer::new(width, height)));
        let (_, fps_rx) = std::sync::mpsc::channel();
        Ok(Self {
            width,
            height,
            front_fb,
            back_fb,
            terminal: None,
            render_thread: None,
            fps_rx,
            fps: 0.0,
            debug,
        })
    }

    /// Rawモードへの変更
    pub fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let fd = io::stdin().as_raw_fd();
        let terminal = term::Terminal::enable(fd)?;
        terminal.set_nonblocking()?;

        term::Terminal::enable_alternative_screen()?;
        term::Terminal::hide_cursor()?;

        self.terminal = Some(terminal);
        Ok(())
    }

    /// 描画スレッドの開始
    pub fn start(&mut self, rate: time::Duration) {
        let (render_thread, fps_rx) =
            render::RenderThread::new(Arc::clone(&self.front_fb), Arc::clone(&self.back_fb), rate);
        self.render_thread = Some(render_thread);
        self.fps_rx = fps_rx;
    }

    /// バッファのMutexの取得
    pub fn get_canvas(&mut self) -> MutexGuard<'_, framebuffer::Framebuffer> {
        let fps = self.get_fps();
        let mut canvas = self.back_fb.lock().unwrap();
        canvas.clear();
        if self.debug {
            canvas.set_str(
                2,
                1,
                &format!("FPS: {:.2}", fps),
                term::Style::default() | term::Color::FgRgb(128, 255, 128),
                framebuffer::Align::Left,
            );
        }
        canvas
    }

    /// ターミナルの復帰
    pub fn end(&mut self) -> io::Result<()> {
        term::Terminal::show_cursor()?;
        term::Terminal::disable_alternative_screen()?;

        self.terminal = None; // Terminalの Drop トレイトが自動的にrawモードを無効化する
        self.render_thread = None; // RenderThreadのDropトレイトが自動的にスレッドを停止する
        Ok(())
    }

    pub fn get_fps(&mut self) -> f64 {
        if let Ok(fps) = self.fps_rx.try_recv() {
            self.fps = fps;
        }
        self.fps
    }
}
