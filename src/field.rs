use crate::block;

pub const FIELD_WIDTH: usize = 10;
pub const FIELD_MARGIN: usize = 4;
pub const FIELD_HEIGHT: usize = 20 + FIELD_MARGIN;

pub type Field = [[Option<block::BlockType>; FIELD_WIDTH]; FIELD_HEIGHT];

pub fn get_field() -> Field {
  [[None; FIELD_WIDTH]; FIELD_HEIGHT]
}

pub trait FieldExt {
  fn is_valid(&self, x: usize, y: usize) -> bool;
  fn set_block(&mut self, block: &block::Block) -> bool;
  fn remove_block(&mut self, block: &block::Block);
  fn get_block(&self, x: usize, y: usize) -> Option<block::BlockType>;
  fn check_collision(&self, block: &block::Block) -> bool;
  fn is_gameover(&self) -> bool;
  fn check_complete_line(&self, y: usize) -> bool;
  fn clear_lines(&mut self) -> u8;
}

impl FieldExt for Field {
  fn is_valid(&self, x: usize, y: usize) -> bool {
    x < FIELD_WIDTH && y < FIELD_HEIGHT
  }

  fn set_block(&mut self, block: &block::Block) -> bool {
    if self.check_collision(block) {
      return false; // Collision detected, do not set the block
    }
    for pos in block.get_relative_positions() {
      let x = pos.x as usize;
      let y = pos.y as usize;
      self[y][x] = Some(block.block_type);
    }
    true
  }

  fn remove_block(&mut self, block: &block::Block) {
    for pos in block.get_relative_positions() {
      let x = pos.x as usize;
      let y = pos.y as usize;
      if self.is_valid(x, y) {
        self[y][x] = None; // Reset to empty
      }
    }
  }

  fn get_block(&self, x: usize, y: usize) -> Option<block::BlockType> {
    if self.is_valid(x, y) {
      self[y][x]
    } else {
      None
    }
  }

  fn check_collision(&self, block: &block::Block) -> bool {
    for pos in block.get_relative_positions() {
      let x = pos.x as usize;
      let y = pos.y as usize;
      if !self.is_valid(x, y) || self.get_block(x, y).is_some() {
        return true;
      }
    }
    false
  }

  fn is_gameover(&self) -> bool {
    for x in 0..FIELD_WIDTH {
      if self[FIELD_MARGIN][x].is_some() {
        return true; // If any block is set in the top row, game over
      }
    }
    false
  }

  fn check_complete_line(&self, y: usize) -> bool {
    self[y].iter().all(|&block| block.is_some())
  }

  fn clear_lines(&mut self) -> u8 {
    let mut cleared = 0;
    for y in 0..FIELD_HEIGHT {
      if self.check_complete_line(y) {
        for i in (1..=y).rev() {
          self[i] = self[i - 1];
        }
        cleared += 1;
      }
    }
    self[0] = [None; FIELD_WIDTH]; // Clear the top line
    cleared
  }
}
