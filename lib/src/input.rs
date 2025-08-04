use std::io::{self, Read};
use std::thread;
use std::time::Duration;
use std::sync::mpsc::{self, Receiver, Sender};

#[derive(Debug, Clone, PartialEq)]
pub enum Key {
  ArrowUp,
  ArrowDown,
  ArrowRight,
  ArrowLeft,
  Home,
  End,
  Insert,
  Delete,
  PageUp,
  PageDown,
  Char(char),
  Escape,
  Unknown,
}

impl Key {
  /// 矢印キーかどうかを判定
  pub fn is_arrow(&self) -> bool {
    matches!(self, Key::ArrowUp | Key::ArrowDown | Key::ArrowLeft | Key::ArrowRight)
  }

  /// 特殊キー（矢印キーやHome/End等）かどうかを判定
  pub fn is_special(&self) -> bool {
    !matches!(self, Key::Char(_) | Key::Unknown)
  }

  /// 印刷可能な文字かどうかを判定
  pub fn is_printable(&self) -> bool {
    matches!(self, Key::Char(c) if c.is_ascii_graphic() || *c == ' ')
  }
}

/// エスケープシーケンスを解析してKeyを返す
fn parse_escape_sequence(buf: &[u8], n: usize) -> Key {
  if n < 3 {
    return Key::Escape;
  }

  match &buf[1..] {
    [b'[', b'A', ..] => Key::ArrowUp,
    [b'[', b'B', ..] => Key::ArrowDown,
    [b'[', b'C', ..] => Key::ArrowRight,
    [b'[', b'D', ..] => Key::ArrowLeft,
    [b'[', b'H', ..] | [b'O', b'H', ..] => Key::Home,
    [b'[', b'F', ..] | [b'O', b'F', ..] => Key::End,
    [b'[', b'2', b'~', ..] => Key::Insert,
    [b'[', b'3', b'~', ..] => Key::Delete,
    [b'[', b'5', b'~', ..] => Key::PageUp,
    [b'[', b'6', b'~', ..] => Key::PageDown,
    _ => Key::Unknown,
  }
}

/// エラーハンドリングを明示的に行うバージョン
fn read_key() -> io::Result<Option<Key>> {
  let mut stdin = io::stdin().lock();
  let mut buf = [0u8; 4];

  match stdin.read(&mut buf) {
    Ok(0) => Ok(None),
    Ok(n) => {
      let key = match buf[0] {
        0x1B => parse_escape_sequence(&buf, n),
        c if c.is_ascii() => Key::Char(c as char),
        _ => Key::Unknown,
      };
      Ok(Some(key))
    }
    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => Ok(None),
    Err(e) => Err(e),
  }
}

pub struct KeyListener {
  pub handle: Option<thread::JoinHandle<()>>,
  pub stop_signal: Option<Sender<()>>,
}

impl KeyListener {
  pub fn new(rate: Duration) -> (Self, Receiver<Key>) {
    let (key_tx, key_rx) = mpsc::channel();
    let (stop_tx, stop_rx) = mpsc::channel();

    let handle = thread::spawn(move || {
      loop {
        if stop_rx.try_recv().is_ok() {
          break; // 停止信号を受け取ったらループを抜ける
        }

        match read_key() {
          Ok(Some(key)) => {
            if key_tx.send(key).is_err() {
              break; // 受信側がドロップされた場合
            }
          }
          Ok(None) => { // キー入力なし
            
          }
          Err(_) => { // 読み取りエラー、継続
            
          }
        }
        thread::sleep(rate);
      }
    });

    (KeyListener { handle: Some(handle), stop_signal: Some(stop_tx) }, key_rx)
  }

  pub fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(tx) = self.stop_signal.take() {
      tx.send(())?; // 停止信号を送信
    }
    if let Some(handle) = self.handle.take() {
      handle.join().map_err(|_| "Failed to join key listener thread")?; // スレッドの終了を待機
    }
    Ok(())
  }
}

impl Drop for KeyListener {
  fn drop(&mut self) {
    let _ = self.stop();
  }
}
