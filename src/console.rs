use crate::framebuffer::FrameBuffer;
use crate::psfparser::psffont;
use core::fmt;
use spin::Mutex;

static CONSOLE: Mutex<Option<Console>> = Mutex::new(None);

pub struct Console {
    framebuffer: FrameBuffer,
    font: &'static psffont,
    x: usize,
    y: usize,
    fg_color: u32,
    bg_color: u32,
    char_width: usize,
    char_height: usize,
    screen_width: usize,
    screen_height: usize,
    margin_l: usize,
    margin_t: usize,
}
impl Console {
    pub fn new(
        framebuffer: FrameBuffer,
        font: &'static psffont,
        fg_color: u32,
        bg_color: u32,
    ) -> Self {
        let screen_width = framebuffer.width();
        let screen_height = framebuffer.height();
        let char_width = font.width();
        let char_height = font.height();
        Self {
            framebuffer,
            font,
            x: 0,
            y: 0,
            fg_color,
            bg_color,
            char_width,
            char_height,
            screen_width,
            screen_height,
            margin_l: 10,
            margin_t: 10,
        }
    }
    pub fn init(frambuffer: FrameBuffer, font: &'static psffont) {
        let mut console = Self::new(frambuffer, font, 0x008000, 0x000000);
        console.clear();
        *CONSOLE.lock() = Some(console);
    }
    pub fn clear(&mut self) {
        self.framebuffer.clear_screen(self.bg_color);
        self.x = 0;
        self.y = 0;
    }
    pub fn write_char(&mut self, ch: char) {
        match ch {
            '\n' => self.newline(),
            '\r' => self.x = 0,
            '\t' => {
                for _ in 0..4 {
                    self.write_char(' ');
                }
            }
            ch => {
                let screen_x = self.margin_l + self.char_width * self.x;
                let screen_y = self.margin_t + self.char_height * self.y;
                if screen_x + self.char_width > self.screen_width - self.margin_l {
                    self.newline();
                    return self.write_char(ch);
                }
                self.framebuffer
                    .draw_char(ch, screen_x, screen_y, self.fg_color, self.font);
                self.x += 1;
            }
        }
    }
    fn newline(&mut self) {
        self.y += 1;
        self.x = 0;
        let max_rows = (self.screen_height - self.margin_t * 2) / self.char_height;
        if self.y >= max_rows {
            self.scroll();
            self.y = max_rows - 1;
        }
    }
    fn scroll(&mut self) {
        let width = self.screen_width;
        let height = self.screen_height;
        for y in self.margin_t..height - self.char_height {
            for x in 0..width {
                let color = self.framebuffer.get_pixel(x, y + self.char_height);
                self.framebuffer.put_pixel(x, y, color);
            }
        }
        let clear_y_start = height - self.char_height - self.margin_t;
        for y in clear_y_start..height {
            for x in 0..width {
                self.framebuffer.put_pixel(x, y, self.bg_color);
            }
        }
    }
    pub fn write_string(&mut self, s: &str) {
        for ch in s.chars() {
            self.write_char(ch);
        }
    }

    pub fn set_fg_color(&mut self, color: u32) {
        self.fg_color = color;
    }

    pub fn set_bg_color(&mut self, color: u32) {
        self.bg_color = color;
    }
}
impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    if let Some(ref mut console) = *CONSOLE.lock() {
        console.write_fmt(args).unwrap();
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
