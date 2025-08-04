#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pos {
  pub x: i32,
  pub y: i32,
}

impl Pos {
  pub fn new(x: i32, y: i32) -> Self {
    Self { x, y }
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockType {
  Empty, I, O, S, Z, J, L, T,
}

impl Default for BlockType {
  fn default() -> Self {
    BlockType::Empty
  }
}

impl BlockType {
  pub fn get_random() -> Self {
    let types = [BlockType::I, BlockType::O, BlockType::S, BlockType::Z, BlockType::J, BlockType::L, BlockType::T];
    let idx = rand::random::<usize>() % types.len();
    types[idx]
  }

  pub fn get_relative_positions(&self) -> [Pos; 3] {
    match self {
      BlockType::Empty => [Pos::new(0, 0); 3],
      BlockType::I => [Pos::new(0, -2), Pos::new(0, -1), Pos::new(0, 1)],
      BlockType::O => [Pos::new(1, 0), Pos::new(0, -1), Pos::new(1, -1)],
      BlockType::S => [Pos::new(-1, 0), Pos::new(0, -1), Pos::new(1, -1)],
      BlockType::Z => [Pos::new(-1, -1), Pos::new(0, -1), Pos::new(1, 0)],
      BlockType::J => [Pos::new(-1, -1), Pos::new(-1, 0), Pos::new(1, 0)],
      BlockType::L => [Pos::new(-1, 0), Pos::new(1, 0), Pos::new(1, -1)],
      BlockType::T => [Pos::new(-1, 0), Pos::new(1, 0), Pos::new(0, -1)],
    }
  }

  pub fn get_color(&self) -> u8 {
    match self {
      BlockType::Empty => 0,
      BlockType::I => 1,
      BlockType::O => 2,
      BlockType::S => 3,
      BlockType::Z => 4,
      BlockType::J => 5,
      BlockType::L => 6,
      BlockType::T => 7,
    }
  }

  pub fn is_empty(&self) -> bool {
    matches!(self, BlockType::Empty)
  }

  pub fn is_block(&self) -> bool {
    !self.is_empty()
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Block {
    pub center_pos: Pos,
    pub block_type: BlockType,
    pub rotation: u8, // 0-3 for 0-270 degrees
}

impl Block {
    pub fn new(center_pos: Pos, block_type: BlockType) -> Self {
        Self {
            center_pos,
            block_type,
            rotation: 0,
        }
    }

    pub fn new_random(center_pos: Pos) -> Self {
        Self {
            center_pos,
            block_type: BlockType::get_random(),
            rotation: 0,
        }
    }

    pub fn init(&mut self, pos: Pos) {
        self.center_pos = pos;
        self.rotation = 0;
    }

    pub fn rotate(&mut self) {
        self.rotation = (self.rotation + 1) % 4;
    }

    pub fn rotate_back(&mut self) {
        self.rotation = (self.rotation + 3) % 4;
    }

    pub fn get_relative_positions(&self) -> [Pos; 4] {
        let relative_positions = self.block_type.get_relative_positions();
        let rotated_relative_positions: [Pos; 3] = relative_positions.map(|pos| {
            let (x, y) = match self.rotation {
                0 => (pos.x, pos.y), // 0 degrees
                1 => (pos.y, -pos.x), // 90 degrees
                2 => (-pos.x, -pos.y), // 180 degrees
                3 => (-pos.y, pos.x), // 270 degrees
                _ => (0, 0), // This should never happen
            };
            Pos::new(self.center_pos.x + x, self.center_pos.y + y)
        });
        [
            self.center_pos,
            rotated_relative_positions[0],
            rotated_relative_positions[1],
            rotated_relative_positions[2],
        ]
    }

    pub fn move_by(&mut self, dx: i32, dy: i32) {
        self.center_pos.x += dx;
        self.center_pos.y += dy;
    }
}
