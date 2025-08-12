use rustui::*;

use crate::block;
use crate::field;
use crate::field::FieldExt;

#[derive(Debug, Clone, PartialEq)]
pub struct Core {
    field: field::Field,
    random_block_pool: Vec<block::BlockType>,
    spawn_pos: block::Pos,
    current_block: block::Block,
    next_block: block::Block,
    holding_block: Option<block::Block>,
    pub field_frame: Framebuffer,
    pub next_block_frame: Framebuffer,
    pub holding_block_frame: Framebuffer,
    drop_counter_max: usize,
    drop_counter: usize,
    moving_after_drop_counter_max: usize,
    moving_after_drop_counter: usize,
    pub is_gameover: bool,
}

impl Core {
    pub fn new(drop_counter_max: usize, moving_after_drop_counter_max: usize) -> Self {
        let mut core = Core {
            field: field::get_field(),
            random_block_pool: Vec::new(),
            spawn_pos: block::Pos::new((field::FIELD_WIDTH / 2) as i32, 2),
            current_block: block::Block::new(block::Pos::new(0, 0), block::BlockType::I),
            next_block: block::Block::new(block::Pos::new(0, 0), block::BlockType::I),
            holding_block: None,
            field_frame: Framebuffer::new(field::FIELD_WIDTH * 2 + 2, field::FIELD_HEIGHT + 2),
            next_block_frame: Framebuffer::new(12, 6),
            holding_block_frame: Framebuffer::new(12, 6),
            drop_counter_max,
            drop_counter: 0,
            moving_after_drop_counter_max,
            moving_after_drop_counter: 0,
            is_gameover: false,
        };
        core.current_block = block::Block::new(
            core.spawn_pos,
            block::BlockType::get_random_from_pool(&mut core.random_block_pool),
        );
        core.next_block = block::Block::new(
            core.spawn_pos,
            block::BlockType::get_random_from_pool(&mut core.random_block_pool),
        );
        core
    }

    fn generate_new_block(&mut self) {
        self.current_block = self.next_block;
        self.current_block.init(self.spawn_pos);
        self.next_block = block::Block::new(
            self.spawn_pos,
            block::BlockType::get_random_from_pool(&mut self.random_block_pool),
        );
    }

    pub fn hold(&mut self) {
        if self.holding_block.is_none() {
            self.holding_block = Some(self.current_block);
            self.generate_new_block();
        } else {
            let held_block = self.holding_block.take().unwrap();
            self.holding_block = Some(self.current_block);
            self.current_block = held_block;
            self.current_block.init(self.spawn_pos);
        }
    }

    pub fn rotate(&mut self) {
        self.current_block.rotate();
        if self.field.check_collision(&self.current_block) {
            self.current_block.rotate_back();
        }
    }

    pub fn move_down(&mut self) {
        self.current_block.move_by(0, 1);
        if self.field.check_collision(&self.current_block) {
            self.current_block.move_by(0, -1);
            if !self.field.set_block(&self.current_block) {
                return;
            }
            self.generate_new_block();
        }
    }

    pub fn quick_drop(&mut self) {
        while !self.field.check_collision(&self.current_block) {
            self.current_block.move_by(0, 1);
        }
        self.current_block.move_by(0, -1);
        if !self.field.set_block(&self.current_block) {
            return;
        }
        self.generate_new_block();
    }

    pub fn move_right(&mut self) {
        self.current_block.move_by(1, 0);
        if self.field.check_collision(&self.current_block) {
            self.current_block.move_by(-1, 0);
        }
    }

    pub fn move_left(&mut self) {
        self.current_block.move_by(-1, 0);
        if self.field.check_collision(&self.current_block) {
            self.current_block.move_by(1, 0);
        }
    }

    fn draw_block_to_buffer(fb: &mut Framebuffer, block: &block::Block) {
        for pos in block.get_relative_positions() {
            fb.set_str(
                pos.x as usize * 2 + 1,
                pos.y as usize + 1,
                "  ",
                Attr::NORMAL,
                (255, 255, 255),
                block.get_color(),
                Align::Left,
            )
        }
    }

    fn update_field_frame(&mut self) {
        self.field_frame.clear();
        self.field_frame
            .set_border(Attr::NORMAL, (255, 255, 255), Color::new());
        self.field_frame.set_str(
            0,
            0,
            "                      ",
            Attr::NORMAL,
            (255, 255, 255),
            Color::new(),
            Align::Left,
        );
        for y in 0..field::FIELD_HEIGHT {
            for x in 0..field::FIELD_WIDTH {
                let color: Color;
                match self.field.get_block(x, y) {
                    Some(block_type) => color = block_type.get_color(),
                    None => color = Color::new(),
                }
                if y == 4 {
                    self.field_frame.set_str(
                        x * 2 + 1,
                        y + 1,
                        "──",
                        Attr::NORMAL,
                        (255, 255, 255),
                        color,
                        Align::Left,
                    );
                    continue;
                }
                self.field_frame.set_str(
                    x * 2 + 1,
                    y + 1,
                    "  ",
                    Attr::NORMAL,
                    (255, 255, 255),
                    color,
                    Align::Left,
                );
            }
        }
    }

    fn update_next_block_frame(&mut self) {
        self.next_block_frame.clear();
        self.next_block_frame.set_named_border(
            "NEXT",
            Align::Center,
            Attr::NORMAL,
            (255, 255, 255),
            Color::new(),
        );
        self.next_block.init(block::Pos::new(2, 2));
        Self::draw_block_to_buffer(&mut self.next_block_frame, &self.next_block);
    }

    fn update_holding_block_frame(&mut self) {
        self.holding_block_frame.clear();
        self.holding_block_frame.set_named_border(
            "HOLD",
            Align::Center,
            Attr::NORMAL,
            (255, 255, 255),
            Color::new(),
        );
        if let &Some(mut block) = &self.holding_block {
            block.init(block::Pos::new(2, 2));
            Self::draw_block_to_buffer(&mut self.holding_block_frame, &block);
        }
    }

    pub fn proc_before_draw(&mut self) {
        if self.drop_counter >= self.drop_counter_max {
            self.drop_counter = 0;
            self.move_down();
        }
        if self.field.is_gameover() {
            self.is_gameover = true;
        }
        self.field.set_block(&self.current_block); // フィールドに現在のブロックをセット
        self.update_field_frame(); // フィールドのフレームバッファを更新
        self.update_holding_block_frame(); // ホールドブロックのフレームバッファを更新
        self.update_next_block_frame(); // 次のブロックのフレームバッファを更新
    }

    pub fn proc_after_draw(&mut self) {
        self.field.remove_block(&self.current_block); // フィールドから現在のブロックを削除
        self.field.clear_lines(); // フィールドのラインをクリア
        self.drop_counter += 1; // ドロップカウンターをインクリメント
        self.moving_after_drop_counter += 1; // 落下後のカウンターをインクリメント
    }
}
