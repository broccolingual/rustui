use rustui::*;
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    thread, time,
};

const RENDERING_RATE: time::Duration = time::Duration::from_millis(31); // ms
const INPUT_CAPTURING_RATE: time::Duration = time::Duration::from_millis(10); // ms

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let file_path = if args.len() > 1 {
        &args[1]
    } else {
        eprintln!("Usage: file_reader <file_path>");
        return Ok(());
    };

    let mut file_reader = FileReader::new(file_path);

    let mut win = Window::new(false)?;
    win.initialize(RENDERING_RATE)?; // Initialize the window and start the rendering thread
    let input_rx = InputListener::new(INPUT_CAPTURING_RATE); // Create an input listener

    loop {
        // Check for key presses
        if let Ok(event) = input_rx.try_recv() {
            match event {
                InputEvent::Key(Key::Char('q')) => break,
                InputEvent::Key(Key::ArrowUp) => {
                    file_reader.scroll_up();
                }
                InputEvent::Key(Key::ArrowDown) => {
                    let visible_lines = win.height.saturating_sub(4);
                    file_reader.scroll_down(visible_lines);
                }
                InputEvent::Key(Key::PageUp) => {
                    let visible_lines = win.height.saturating_sub(4);
                    file_reader.page_up(visible_lines);
                }
                InputEvent::Key(Key::PageDown) => {
                    let visible_lines = win.height.saturating_sub(4);
                    file_reader.page_down(visible_lines);
                }
                InputEvent::Key(Key::Home) => {
                    file_reader.current_line = 0;
                    file_reader.scroll_offset = 0;
                }
                InputEvent::Key(Key::End) => {
                    let visible_lines = win.height.saturating_sub(4);
                    file_reader.current_line = file_reader.lines.len().saturating_sub(1);
                    file_reader.scroll_offset =
                        file_reader.lines.len().saturating_sub(visible_lines);
                }
                _ => {}
            }
        }

        // Draw the frame
        win.draw(|canvas| {
            file_reader.render(canvas);
        })?;

        thread::sleep(time::Duration::from_millis(31)); // Sleep to prevent high CPU usage
    }
    Ok(())
}

struct FileReader {
    file_path: String,
    lines: Vec<String>,
    current_line: usize,
    scroll_offset: usize,
}

impl FileReader {
    fn new(file_path: &str) -> Self {
        let file = File::open(file_path).unwrap();
        let reader = BufReader::new(file);
        let lines: Result<Vec<String>, io::Error> = reader.lines().collect();
        Self {
            file_path: file_path.to_string(),
            lines: lines.unwrap(),
            current_line: 0,
            scroll_offset: 0,
        }
    }

    fn scroll_up(&mut self) {
        if self.current_line > 0 {
            self.current_line -= 1;
            if self.current_line < self.scroll_offset {
                self.scroll_offset = self.current_line;
            }
        }
    }

    fn scroll_down(&mut self, visible_lines: usize) {
        if self.current_line + 1 < self.lines.len() {
            self.current_line += 1;
            if self.current_line >= self.scroll_offset + visible_lines {
                self.scroll_offset = self.current_line - visible_lines + 1;
            }
        }
    }

    fn page_up(&mut self, visible_lines: usize) {
        let page_size = visible_lines.saturating_sub(1);
        self.current_line = self.current_line.saturating_sub(page_size);
        self.scroll_offset = self.scroll_offset.saturating_sub(page_size);
    }

    fn page_down(&mut self, visible_lines: usize) {
        let page_size = visible_lines.saturating_sub(1);
        self.current_line = (self.current_line + page_size).min(self.lines.len().saturating_sub(1));

        if self.current_line >= self.scroll_offset + visible_lines {
            self.scroll_offset = self.current_line - visible_lines + 1;
        }
    }

    fn render(&self, canvas: &mut Framebuffer) {
        // ボーダーを設定（ファイル名を表示）
        canvas.set_named_border(
            &format!("File Reader - {}", self.file_path),
            Align::Center,
            Attr::default(),
            Color::White,
            Color::default(),
        );

        // コンテンツ表示領域のサイズを計算（ボーダーを除く）
        let content_width = canvas.width.saturating_sub(4);
        let content_height = canvas.height.saturating_sub(4);
        let start_x = 2;
        let start_y = 2;

        // ファイルの内容を表示
        for (i, line_idx) in (self.scroll_offset..self.scroll_offset + content_height).enumerate() {
            if line_idx >= self.lines.len() {
                break;
            }

            let line = &self.lines[line_idx];
            let is_current = line_idx == self.current_line;

            // 現在行のハイライト
            let (attrs, fg, bg) = if is_current {
                (Attr::default(), Color::Black, Color::White)
            } else {
                (Attr::default(), Color::White, Color::default())
            };

            // 行番号を表示
            let line_number = format!("{:4} ", line_idx + 1);
            canvas.set_str(
                start_x,
                start_y + i,
                &line_number,
                attrs,
                Color::Cyan,
                bg,
                Align::Left,
            );

            // ファイルの内容を表示（長い行は切り詰める）
            let display_line = if line.len() > content_width - 5 {
                format!("{}...", &line[..content_width - 8])
            } else {
                line.clone()
            };

            canvas.set_str(
                start_x + 5,
                start_y + i,
                &display_line,
                attrs,
                fg,
                bg,
                Align::Left,
            );
        }

        // ステータスバーを表示
        let status = format!(
            "Line {}/{} | Press ↑↓ to scroll, PgUp/PgDn for pages, 'q' to quit",
            self.current_line + 1,
            self.lines.len()
        );

        canvas.set_str(
            canvas.width / 2,
            canvas.height - 1,
            &status,
            Attr::default(),
            Color::Yellow,
            Color::default(),
            Align::Center,
        );
    }
}
