use rand::Rng;
use rustui::*;

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
    I,
    O,
    S,
    Z,
    J,
    L,
    T,
}

impl BlockType {
    pub fn get_random_from_pool(pool: &mut Vec<BlockType>) -> Self {
        if pool.is_empty() {
            pool.extend(vec![
                BlockType::I,
                BlockType::O,
                BlockType::S,
                BlockType::Z,
                BlockType::J,
                BlockType::L,
                BlockType::T,
            ]);
        }
        let idx = rand::rng().random_range(0..pool.len());
        pool.remove(idx)
    }

    pub fn get_relative_positions(&self) -> [Pos; 3] {
        match self {
            BlockType::I => [Pos::new(0, -2), Pos::new(0, -1), Pos::new(0, 1)],
            BlockType::O => [Pos::new(1, 0), Pos::new(0, -1), Pos::new(1, -1)],
            BlockType::S => [Pos::new(-1, 0), Pos::new(0, -1), Pos::new(1, -1)],
            BlockType::Z => [Pos::new(-1, -1), Pos::new(0, -1), Pos::new(1, 0)],
            BlockType::J => [Pos::new(-1, -1), Pos::new(-1, 0), Pos::new(1, 0)],
            BlockType::L => [Pos::new(-1, 0), Pos::new(1, 0), Pos::new(1, -1)],
            BlockType::T => [Pos::new(-1, 0), Pos::new(1, 0), Pos::new(0, -1)],
        }
    }

    pub fn get_color(&self) -> Color {
        match self {
            BlockType::I => (0, 255, 255),
            BlockType::O => (255, 255, 0),
            BlockType::S => (0, 255, 0),
            BlockType::Z => (255, 0, 0),
            BlockType::J => (0, 0, 255),
            BlockType::L => (255, 165, 0),
            BlockType::T => (128, 0, 128),
        }
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
        if self.block_type == BlockType::O {
            // Oブロックは回転しないのでそのまま返す
            return [
                self.center_pos,
                Pos::new(
                    self.center_pos.x + relative_positions[0].x,
                    self.center_pos.y + relative_positions[0].y,
                ),
                Pos::new(
                    self.center_pos.x + relative_positions[1].x,
                    self.center_pos.y + relative_positions[1].y,
                ),
                Pos::new(
                    self.center_pos.x + relative_positions[2].x,
                    self.center_pos.y + relative_positions[2].y,
                ),
            ];
        }
        let rotated_relative_positions: [Pos; 3] = relative_positions.map(|pos| {
            let (x, y) = match self.rotation {
                0 => (pos.x, pos.y),   // 0 degrees
                1 => (-pos.y, pos.x),  // 90 degrees
                2 => (-pos.x, -pos.y), // 180 degrees
                3 => (pos.y, -pos.x),  // 270 degrees
                _ => (0, 0),           // This should never happen
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

    pub fn get_color(&self) -> Color {
        self.block_type.get_color()
    }
}
