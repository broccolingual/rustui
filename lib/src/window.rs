use std::io;
use std::os::unix::io::AsRawFd;
use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::mpsc::{Receiver};
use std::time;

use crate::render;
use crate::term;
use crate::framebuffer;

pub struct Window {
  pub width: usize,
  pub height: usize,
  pub front_fb: Arc<Mutex<framebuffer::Framebuffer>>,
  pub back_fb: Arc<Mutex<framebuffer::Framebuffer>>,
  pub terminal: Option<term::Terminal>,
  pub render_thread: Option<render::RenderThread>,
}

impl Window {
  /// コンストラクタ
  pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
    let (width, height) = term::Terminal::get_size()?;
    let front_fb = Arc::new(Mutex::new(framebuffer::Framebuffer::new(width, height)));
    let back_fb = Arc::new(Mutex::new(framebuffer::Framebuffer::new(width, height)));
    Ok(Self { width, height, front_fb, back_fb, terminal: None, render_thread: None })
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
  pub fn start(&mut self, rate: time::Duration) -> Receiver<f64> {
    let (render_thread, fps_rx) = render::RenderThread::new(
      Arc::clone(&self.front_fb),
      Arc::clone(&self.back_fb),
      rate,
    );
    self.render_thread = Some(render_thread);
    fps_rx
  }

  /// バッファのMutexの取得
  pub fn get_lock(&mut self) -> MutexGuard<'_, framebuffer::Framebuffer> {
    self.back_fb.lock().unwrap()
  }

  /// ターミナルの復帰
  pub fn end(&mut self) -> io::Result<()> {
    
    term::Terminal::show_cursor()?;
    term::Terminal::disable_alternative_screen()?;
    
    self.terminal = None; // Terminalの Drop トレイトが自動的にrawモードを無効化する
    self.render_thread = None; // RenderThreadのDropトレイトが自動的にスレッドを停止する
    Ok(())
  }
}
