use core::panic;
use std::{thread, time};
use rcurses::*;

pub mod field;
pub mod block;

use block::*;
use field::*;

const RENDERING_RATE: time::Duration = time::Duration::from_millis(15); // ms
const INPUT_CAPTURING_RATE: time::Duration = time::Duration::from_millis(15); // ms

fn main() {
  let mut win = Window::new().expect("Failed to create window");
  win.init().expect("Failed to initialize terminal");
  let fps_rx = win.start(RENDERING_RATE);
  let (mut key_listener, key_rx) = KeyListener::new(INPUT_CAPTURING_RATE);

  if win.width < 60 || win.height < 30 {
      key_listener.stop().expect("Failed to stop key listener");
      win.end().expect("Failed to cleanup terminal");
      panic!("Window size is too small. Minimum size is 60x30.");
  }

  let x_center = win.width / 2;
  let y_center: usize = win.height / 2;

  let mut fps: f64 = 0.0;
  let mut last_key = None;
  let mut info_frame = Framebuffer::new(32, 2);

  {
    let mut back_locked = win.get_lock();
    back_locked.set_str(x_center, y_center - 3, "R U S T", style![Attr::BOLD, Color::Fg24(96)], Align::Center);
    back_locked.set_str(x_center, y_center - 2, "T E T R I S", style![Attr::BOLD, Color::Fg24(96)], Align::Center);
    back_locked.set_border(style![Attr::NORMAL]);
  }

  thread::sleep(time::Duration::from_secs(1));

  let mut field_frame = Framebuffer::new(FIELD_WIDTH * 2 + 2, FIELD_HEIGHT + 2);
  let mut holding_frame = Framebuffer::new(12, 6);

  let init_pos = Pos::new((FIELD_WIDTH / 2) as i32, 2);
  let mut field = get_field();
  let mut random_block_pool = vec![];
  let mut current_block = Block::new(init_pos, BlockType::get_random_from_pool(&mut random_block_pool));
  let mut holding_block: Option<Block> = None;
  let drop_counter_max = 100; // Adjust this value to control the drop speed
  let mut drop_counter = 0;
  let mut is_gameover = false;

  loop {
    if let Ok(v) = fps_rx.try_recv() {
      fps = v;
    }

    info_frame.clear();
    info_frame.set_str(1, 0, &format!("FPS: {:.2}", &fps), style![Attr::ITALIC, Attr::NORMAL, Color::Fg24(74)], Align::Left);
    info_frame.set_str(1, 1, &format!("Key: {:?}", last_key), style![Attr::ITALIC, Attr::NORMAL, Color::Fg24(64)], Align::Left);
    info_frame.set_color(Color::Bg24(235));

    if let Ok(key) = key_rx.try_recv() {
      last_key = Some(key.clone());
      match key {
        Key::Char('q') => break,
        Key::Char('r') => {
          if holding_block.is_none() {
            current_block.init(init_pos);
            holding_block = Some(current_block);
            current_block = Block::new(init_pos, BlockType::get_random_from_pool(&mut random_block_pool));
          } else {
            let temp = holding_block.take().unwrap();
            holding_block = Some(current_block);
            current_block = temp;
            current_block.init(init_pos);
          }
        }
        Key::Char(' ') => {
          current_block.rotate();
          if field.check_collision(&current_block) {
            current_block.rotate_back();
          }
        }
        Key::ArrowUp => {
        }
        Key::ArrowDown => {
          current_block.move_by(0, 1);
          if field.check_collision(&current_block) {
            current_block.move_by(0, -1);
            if !field.set_block(&current_block) {
              break;
            }
            current_block = Block::new(init_pos, BlockType::get_random_from_pool(&mut random_block_pool));
          }
        }
        Key::ArrowRight => {
          current_block.move_by(1, 0);
          if field.check_collision(&current_block) {
            current_block.move_by(-1, 0);
          }
        }
        Key::ArrowLeft => {
          current_block.move_by(-1, 0);
          if field.check_collision(&current_block) {
            current_block.move_by(1, 0);
          }
        }
        _ => (),
      }
    }

    if drop_counter >= drop_counter_max {
      drop_counter = 0;
      current_block.move_by(0, 1);
      if field.check_collision(&current_block) {
        current_block.move_by(0, -1);
        if !field.set_block(&current_block) {
          break;
        }
        current_block = Block::new(init_pos, BlockType::get_random_from_pool(&mut random_block_pool));
      }
    }
    
    if field.is_gameover() {
      is_gameover = true;
      break;
    }

    field.set_block(&current_block);
    
    // フィールドのフレームバッファを更新
    field_frame.clear();
    field_frame.set_border(style![Attr::NORMAL]);
    field_frame.set_color(Color::Bg(0));
    field_frame.set_str(0, 0, "                      ", style![Attr::BOLD], Align::Left);
    for y in 0..FIELD_HEIGHT {
      for x in 0..FIELD_WIDTH {
        let block_type = field.get_block(x, y);
        if y == 4 {
          field_frame.set_str(x * 2 + 1, y + 1, "--", style![Attr::THIN, Color::Bg(block_type.get_color())], Align::Left);
          continue;
        }
        field_frame.set_str(x * 2 + 1, y + 1, "  ", style![Attr::NORMAL, Color::Bg(block_type.get_color())], Align::Left);
      }
    }

    // ホールドブロックのフレームバッファを更新
    holding_frame.clear();
    holding_frame.set_border(style![Attr::NORMAL]);
    holding_frame.set_color(Color::Bg(0));
    holding_frame.set_str(holding_frame.width / 2, 0, "HOLD", style![Attr::BOLD], Align::Center);
    if let Some(mut block) = holding_block {
      block.init(Pos::new(2, 2));
      for pos in block.get_relative_positions() {
        holding_frame.set_str(pos.x as usize * 2 + 1, pos.y as usize + 1, "  ", style![Attr::NORMAL, Color::Bg(block.block_type.get_color())], Align::Left);
      }
    }

    {
      let mut back_locked = win.get_lock();
      back_locked.clear();
      back_locked.set_border(style![Attr::NORMAL]);
      back_locked.combine(&field_frame, x_center - field_frame.width / 2, y_center - field_frame.height / 2);
      back_locked.combine(&holding_frame, x_center + field_frame.width / 2 + 2, y_center - field_frame.height / 2 + 1);
      back_locked.combine(&info_frame, 2, 1);
    }

    thread::sleep(time::Duration::from_millis(5));

    field.remove_block(&current_block);
    field.clear_lines();

    drop_counter += 1;
  }

  if is_gameover {
    {
      let mut back_locked = win.get_lock();
      back_locked.clear();
      back_locked.set_border(style![Attr::NORMAL]);
      back_locked.set_str(x_center, y_center, "G A M E  O V E R", style![Attr::BOLD, Color::Fg24(196)], Align::Center);
    }
    thread::sleep(time::Duration::from_secs(3));
  }

  key_listener.stop().expect("Failed to stop key listener");
  win.end().expect("Failed to cleanup terminal");
}
