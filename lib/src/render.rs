use std::sync::{Arc, Mutex, mpsc::{self, Sender, Receiver}};
use std::sync::TryLockError;
use std::thread;
use std::time;

use crate::framebuffer;

pub struct RenderThread {
  pub handle: Option<thread::JoinHandle<()>>,
  pub stop_signal: Option<Sender<()>>,
}

impl RenderThread {
  pub fn new(
    front_fb: Arc<Mutex<framebuffer::Framebuffer>>,
    back_fb: Arc<Mutex<framebuffer::Framebuffer>>,
    rate: time::Duration,
  ) -> (Self, Receiver<f64>) {
    let (fps_tx, fps_rx): (Sender<f64>, Receiver<f64>) = mpsc::channel();
    let (stop_tx, stop_rx) = mpsc::channel();

    let handle = thread::spawn(move || {
      let mut last_sec = time::Instant::now();
      let mut frame_count = 0;
      let mut last_frame_time = time::Instant::now();

      loop {
        if stop_rx.try_recv().is_ok() {
          break; // 停止信号を受け取ったらループを抜ける
        }

        // フレームレート制御
        let elapsed_since_frame = last_frame_time.elapsed();
        if elapsed_since_frame < rate {
          thread::sleep(rate - elapsed_since_frame);
        }
        last_frame_time = time::Instant::now();

        // 描画処理
        match back_fb.try_lock() {
          Ok(back_locked) => {
            if let Ok(mut front_locked) = front_fb.try_lock() {
              if front_locked.refresh(&back_locked).is_err() {
                continue;
              }
              frame_count += 1;
            }
          }
          Err(TryLockError::WouldBlock) => { // バックバッファがロックされている場合はスキップ
            thread::sleep(time::Duration::from_millis(1));
            continue;
          }
          Err(_) => {
            break;
          }
        }

        // FPS計算と送信
        let elapsed = last_sec.elapsed();
        if elapsed >= time::Duration::from_secs(1) {
          let fps = frame_count as f64 / elapsed.as_secs_f64();
          if fps_tx.send(fps).is_err() { // 受信側が閉じられた場合は終了
            break;
          }
          frame_count = 0;
          last_sec = time::Instant::now();
        }
      }
    });

    (
      RenderThread {
        handle: Some(handle),
        stop_signal: Some(stop_tx)
      }, fps_rx
    )
  }

  pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(tx) = self.stop_signal.take() {
      tx.send(())?; // 停止信号を送信
    }
    if let Some(handle) = self.handle.take() {
      handle.join().map_err(|_| "Failed to join render thread")?; // スレッドの終了を待機
    }
    Ok(())
  }
}

impl Drop for RenderThread {
  fn drop(&mut self) {
    let _ = self.stop();
  }
}
