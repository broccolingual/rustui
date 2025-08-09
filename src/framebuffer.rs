use std::io::{self, Write};

use crate::term;

#[derive(Clone, PartialEq, Debug)]
pub struct Cell {
    pub ch: char,
    pub style: term::Style,
}

impl Cell {
    /// コンストラクタ
    pub fn new() -> Self {
        Self {
            ch: ' ',
            style: term::Style::default(),
        }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::new()
    }
}

pub enum Align {
    Left,
    Center,
    Right,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    buffer: Vec<Vec<Cell>>,
}

impl Framebuffer {
    /// コンストラクタ
    pub fn new(width: usize, height: usize) -> Self {
        let buffer = vec![vec![Cell::default(); width]; height];
        Self {
            width,
            height,
            buffer,
        }
    }

    /// バッファを初期化
    pub fn clear(&mut self) {
        for row in &mut self.buffer {
            for cell in row {
                *cell = Cell::default();
            }
        }
    }

    /// バッファの全てのセルに色を設定
    pub fn set_color_all(&mut self, color: term::Color) {
        for row in &mut self.buffer {
            for cell in row {
                cell.style = cell.style | color;
            }
        }
    }

    /// x, yがフレームバッファのサイズに収まっているかをチェック
    pub fn check_range(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height
    }

    /// バッファに文字と属性を書き込み
    pub fn set_char(&mut self, x: usize, y: usize, ch: char, style: term::Style) {
        if self.check_range(x, y) {
            self.buffer[y][x].ch = ch;
            self.buffer[y][x].style = style;
        }
    }

    /// バッファに文字列と属性を書き込み
    pub fn set_str(&mut self, x: usize, y: usize, str: &str, style: term::Style, align: Align) {
        let start_x = match align {
            Align::Left => x,
            Align::Center => {
                if str.len() > x {
                    0
                } else {
                    x - str.len() / 2
                }
            }
            Align::Right => {
                if str.len() > x {
                    0
                } else {
                    x - str.len()
                }
            }
        };
        for (i, ch) in str.chars().enumerate() {
            self.set_char(start_x + i, y, ch, style.clone());
        }
    }

    /// バッファのサイズの外枠を描画
    pub fn set_border(&mut self, style: term::Style) {
        let w = self.width;
        let h = self.height;

        // 上下の線
        for x in 1..w - 1 {
            self.set_char(x, 0, '─', style.clone());
            self.set_char(x, h - 1, '─', style.clone());
        }

        // 左右の線
        for y in 1..h - 1 {
            self.set_char(0, y, '│', style.clone());
            self.set_char(w - 1, y, '│', style.clone());
        }

        // 角の文字
        self.set_char(0, 0, '┌', style.clone());
        self.set_char(w - 1, 0, '┐', style.clone());
        self.set_char(0, h - 1, '└', style.clone());
        self.set_char(w - 1, h - 1, '┘', style.clone());
    }

    /// バッファの内容を結合
    pub fn combine(&mut self, other: &Framebuffer, x_offset: usize, y_offset: usize) {
        for y in 0..other.height {
            for x in 0..other.width {
                if self.check_range(x + x_offset, y + y_offset) {
                    let cell = &other.buffer[y][x];
                    self.set_char(x + x_offset, y + y_offset, cell.ch, cell.style.clone());
                }
            }
        }
    }

    /// バックバッファとフロントバッファを比較し，差分を描画．フロントバッファをバックバッファの内容で更新
    pub fn refresh(&mut self, back_fb: &Framebuffer) -> io::Result<()> {
        if self.height != back_fb.height || self.width != back_fb.width {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Framebuffer sizes do not match",
            ));
        }

        let mut output = String::new();
        let mut prev_attr: Option<&term::Style> = None;

        for y in 0..self.height {
            for x in 0..self.width {
                let front = &self.buffer[y][x];
                let back = &back_fb.buffer[y][x];

                if front != back {
                    // フロントバッファのCellとバックバッファのCellが異なる場合のみ描画
                    if prev_attr != Some(&back.style) {
                        // 属性が異なる場合のみ属性を更新
                        output.push_str("\x1B[0m"); // 属性をリセット
                        output.push_str(&back.style.to_ansi()); // 属性を付与
                        prev_attr = Some(&back.style); // 属性を更新
                    }
                    output.push_str(&format!("\x1B[{};{}H", y + 1, x + 1)); // 対象の座標に移動
                    output.push(back.ch); // 文字を追加
                    self.buffer[y][x] = back.clone(); // フロントバッファにCellをコピー
                }
            }
        }

        output.push_str(&term::Style::default().to_ansi()); // 属性をリセット
        io::stdout().write_all(output.as_bytes())?; // バッファに書き込む
        io::stdout().flush()?; // バッファを描画
        Ok(())
    }
}
