pub struct psffont {
    width: usize,
    height: usize,
    char_size: usize,
    number_of_char: usize,
    char_data: &'static [u8],
}
impl psffont {
    pub fn parse(data: &'static [u8]) -> Result<Self, &'static str> {
        if data.len() < 4 {
            return Err("Too Small");
        }
        if data.len() >= 32
            && data[0] == 0x72
            && data[1] == 0xb5
            && data[2] == 0x4a
            && data[3] == 0x86
        {
            return Self::parsepsf2(data);
        }
        if data[0] == 0x36 && data[1] == 0x04 {
            return Self::parsepsf1(data);
        }
        Err("Invalid header")
    }
    fn parsepsf1(data: &'static [u8]) -> Result<Self, &'static str> {
        let mode = data[2];
        let char_size = data[3] as usize;
        let width = 8;
        let height = char_size;
        let num_of_chars = if mode & 0x01 != 0 { 512 } else { 256 };
        let header_size = 4;
        Ok(Self {
            width,
            height,
            char_size,
            number_of_char: num_of_chars,
            char_data: &data[header_size..],
        })
    }
    fn parsepsf2(data: &'static [u8]) -> Result<Self, &'static str> {
        if data.len() < 32 {
            return Err("Header too small");
        }
        let header_size = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;
        let num_glyphs = u32::from_le_bytes([data[16], data[17], data[18], data[19]]) as usize;
        let glyph_size = u32::from_le_bytes([data[20], data[21], data[22], data[23]]) as usize;
        let height = u32::from_le_bytes([data[24], data[25], data[26], data[27]]) as usize;
        let width = u32::from_le_bytes([data[28], data[29], data[30], data[31]]) as usize;

        if data.len() < header_size + (num_glyphs * glyph_size) {
            return Err("PSF2 data too small for declared glyphs");
        }

        Ok(Self {
            width,
            height,
            char_size: glyph_size,
            number_of_char: num_glyphs,
            char_data: &data[header_size..],
        })
    }
    pub fn get_char(&self, ch: char) -> Option<&[u8]> {
        let index = ch as usize;
        if index >= self.number_of_char {
            let fallback = if self.number_of_char > '?' as usize {
                '?' as usize
            } else {
                0
            };
            let start = fallback * self.char_size;
            let end = start + self.char_size;
            return Some(&self.char_data[start..end]);
        }
        let start = index * self.char_size;
        let end = start + self.char_size;
        return Some(&self.char_data[start..end]);
    }
    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
    pub fn is_pixel_set(&self, glyph: &[u8], x: usize, y: usize) -> bool {
        if y >= self.height || x >= self.width {
            return false;
        }

        // Calculate bytes per row (rounded up)
        let bytes_per_row = (self.width + 7) / 8;
        let byte_index = y * bytes_per_row + (x / 8);

        if byte_index >= glyph.len() {
            return false;
        }

        let bit_index = 7 - (x % 8);
        (glyph[byte_index] >> bit_index) & 1 != 0
    }
}
