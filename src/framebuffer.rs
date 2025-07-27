use std::io::{self, Write};

use crate::term;

#[derive(Clone, PartialEq)]
pub struct Cell {
  pub ch: char,
  pub attr: term::Attr,
}

impl Cell {
  pub fn new() -> Self {
    let ch: char = ' ';
    let attr = term::Attr::Normal;
    Self { ch, attr }
  }
}

pub struct Framebuffer {
  pub width: usize,
  pub height: usize,
  pub buffer: Vec<Vec<Cell>>,
}

impl Framebuffer {
  pub fn new(width: usize, height: usize) -> Self {
    let cell = Cell::new();
    let buffer = vec![vec![cell; width]; height]; 
    Self { width, height, buffer }
  }

  pub fn clear(&mut self) {
    for y in 0..self.height {
      for x in 0..self.width {
        self.buffer[y][x] = Cell::new(); 
      }
    }
  }

  pub fn check_range(&self, x: &usize, y: &usize) -> bool {
    if x >= &0 && x < &self.width && y >= &0 && y < &self.height {
      true
    } else {
      false
    }
  }

  pub fn set_char(&mut self, x: &usize, y: &usize, ch: char, attr: term::Attr) {
    if self.check_range(x, y) {
      self.buffer[*y][*x].ch = ch;
      self.buffer[*y][*x].attr = attr;
    }
  }

  pub fn set_str(&mut self, x: &usize, y: &usize, str: &String, attr: term::Attr) {
    for (i, ch) in str.chars().enumerate() {
      self.set_char(&(x + i), y, ch, attr.clone());
    }
  }

  pub fn set_border(&mut self, attr: term::Attr) {
    let w = self.width;
    let h = self.height;

    for x in 1..w - 1 {
      self.set_char(&x, &0, '─', attr.clone());
      self.set_char(&x, &(h - 1), '─', attr.clone());
    }

    for y in 1..h - 1 {
      self.set_char(&0, &y, '│', attr.clone());
      self.set_char(&(w - 1), &y, '│', attr.clone());
    }

    self.set_char(&0, &0, '┌', attr.clone());
    self.set_char(&(w - 1) , &0, '┐', attr.clone());
    self.set_char(&0, &(h - 1), '└', attr.clone());
    self.set_char(&(w - 1), &(h - 1), '┘', attr.clone());
  }

  pub fn refresh(&mut self, back_fb: &Framebuffer) {
    let mut output = String::new();
    let mut prev_attr: Option<&term::Attr> = None;

    for y in 0..self.height {
      for x in 0..self.width {
        let front = &self.buffer[y][x];
        let back = &back_fb.buffer[y][x];
        
        if front != back {
          if prev_attr != Some(&back.attr) {
            output.push_str("\x1B[0m"); // 属性リセット
            output.push_str(&term::attr_to_ansi(&back.attr)); // 属性を付与
            prev_attr = Some(&back.attr)
          }
          output.push_str(&format!("\x1B[{};{}H", y + 1, x + 1)); // 対象の座標に移動
          output.push(back.ch); // 文字を追加
          self.buffer[y][x] = back.clone();
        }
      }
    }

    output.push_str("\x1B[0m");
    io::stdout().write_all(output.as_bytes()).unwrap();
    io::stdout().flush().unwrap();
  }
}
