use crate::psfparser::psffont;
use uefi::proto::console::gop::PixelFormat;
pub struct FrameBufferInfo {
    pub addr: usize,
    pub size: usize,
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub pixel_format: PixelFormat,
}
pub struct FrameBuffer {
    buffer: &'static mut [u8],
    width: usize,
    height: usize,
    stride: usize,
    pixel_format: PixelFormat,
    size_per_pixel: usize,
}
impl FrameBuffer {
    pub fn new(info: FrameBufferInfo) -> Self {
        let pix_size = 4;
        let buffer = unsafe { core::slice::from_raw_parts_mut(info.addr as *mut u8, info.size) };
        Self {
            buffer,
            width: info.width,
            height: info.height,
            stride: info.stride,
            pixel_format: info.pixel_format,
            size_per_pixel: pix_size,
        }
    }
    pub fn clear_screen(&mut self, color: u32) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.put_pixel(x, y, color);
            }
        }
    }
    pub fn put_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x >= self.width || y >= self.height {
            return;
        }
        let pixel = y * self.stride + x;
        let offset = self.size_per_pixel * pixel;
        let (r, g, b) = (
            ((color >> 16) & 0xFF) as u8,
            ((color >> 8) & 0xFF) as u8,
            (color & 0xFF) as u8,
        );
        if offset + 3 >= self.buffer.len() {
            return;
        }
        match self.pixel_format {
            PixelFormat::Rgb => {
                self.buffer[offset] = r;
                self.buffer[offset + 1] = g;
                self.buffer[offset + 2] = b;
                self.buffer[offset + 3] = 0;
            }
            PixelFormat::Bgr => {
                self.buffer[offset] = b;
                self.buffer[offset + 1] = g;
                self.buffer[offset + 2] = r;
                self.buffer[offset + 3] = 0;
            }
            _ => {
                self.buffer[offset] = b;
                self.buffer[offset + 1] = g;
                self.buffer[offset + 2] = r;
                self.buffer[offset + 3] = 0;
            }
        }
    }

    pub fn draw_char(&mut self, ch: char, x: usize, y: usize, color: u32, font: &psffont) {
        if let Some(glyph) = font.get_char(ch) {
            for row in 0..font.height() {
                for col in 0..font.width() {
                    if font.is_pixel_set(glyph, col, row) {
                        self.put_pixel(x + col, y + row, color);
                    }
                }
            }
        }
    }
    pub fn write_string(&mut self, text: &str, mut x: usize, y: usize, color: u32, font: &psffont) {
        for ch in text.chars() {
            if ch == '\n' {
                continue;
            }
            self.draw_char(ch, x, y, color, font);
            x += font.width();
        }
    }
    pub fn draw_rect(&mut self, x: usize, y: usize, width: usize, height: usize, color: u32) {
        for dy in 0..height {
            for dx in 0..width {
                self.put_pixel(x + dx, y + dy, color);
            }
        }
    }
    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
    pub fn get_pixel(&self, x: usize, y: usize) -> u32 {
        if x >= self.width || y >= self.height {
            return 0;
        }

        let pixel_offset = y * self.stride + x;
        let byte_offset = pixel_offset * self.size_per_pixel;

        if byte_offset + 3 >= self.buffer.len() {
            return 0;
        }

        match self.pixel_format {
            PixelFormat::Rgb => {
                let r = self.buffer[byte_offset] as u32;
                let g = self.buffer[byte_offset + 1] as u32;
                let b = self.buffer[byte_offset + 2] as u32;
                (r << 16) | (g << 8) | b
            }
            PixelFormat::Bgr => {
                let b = self.buffer[byte_offset] as u32;
                let g = self.buffer[byte_offset + 1] as u32;
                let r = self.buffer[byte_offset + 2] as u32;
                (r << 16) | (g << 8) | b
            }
            _ => {
                let b = self.buffer[byte_offset] as u32;
                let g = self.buffer[byte_offset + 1] as u32;
                let r = self.buffer[byte_offset + 2] as u32;
                (r << 16) | (g << 8) | b
            }
        }
    }
}
