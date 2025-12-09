pub struct BitReader<'a> {
    data: &'a [u8],
    byte_position: usize,
    bit_position: u8,
}

impl<'a> BitReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            byte_position: 0,
            bit_position: 0,
        }
    }

    pub fn read_bit(&mut self) -> Option<bool> {
        if self.byte_position >= self.data.len() {
            return None;
        }

        let byte = self.data[self.byte_position];
        let bit = (byte & (1 << (7 - self.bit_position))) != 0;

        self.bit_position += 1;
        if self.bit_position >= 8 {
            self.bit_position = 0;
            self.byte_position += 1;
        }

        Some(bit)
    }

    pub fn read_n_bits(&mut self, count: u32) -> Option<u32> {
        let mut result = 0u32;
        for _ in 0..count {
            let bit = self.read_bit()? as u32;
            result = (result << 1) | bit;
        }
        Some(result)
    }
}

pub struct BitWriter {
    data: Vec<u8>,
    current_byte: u8,
    bit_position: u8,
}

impl BitWriter {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            current_byte: 0,
            bit_position: 0,
        }
    }

    pub fn write_bit(&mut self, bit: bool) {
        if bit {
            self.current_byte |= 1 << (7 - self.bit_position);
        }

        self.bit_position += 1;
        if self.bit_position >= 8 {
            self.data.push(self.current_byte);
            self.current_byte = 0;
            self.bit_position = 0;
        }
    }

    pub fn write_n_bits(&mut self, count: u32, value: u32) {
        for i in (0..count).rev() {
            let bit = (value >> i) & 1 == 1;
            self.write_bit(bit);
        }
    }

    pub fn flush(&mut self) {
        if self.bit_position > 0 {
            self.data.push(self.current_byte);
            self.current_byte = 0;
            self.bit_position = 0;
        }
    }

    pub fn finish(mut self) -> Vec<u8> {
        self.flush();
        self.data
    }
}
