use crate::bit_operations::{BitReader, BitWriter};
use crate::models::Token;
use std::cmp::min;

pub struct LZ77 {
    offset_bits: u8,
    length_bits: u8,
    max_offset: usize,
    max_length: usize,
    window: Vec<u8>,
    lab_start_index: usize,
    tokens: Vec<Token>,
}

impl LZ77 {
    pub fn new(offset_bits: Option<u8>, length_bits: Option<u8>) -> Self {
        let (offset_bits, length_bits) = (offset_bits.unwrap_or(0), length_bits.unwrap_or(0));
        let mut max_offset = 0;
        let mut max_length = 0;

        if offset_bits > 0 {
            max_offset = (1 << offset_bits) - 1;
        }

        if length_bits > 0 {
            max_length = (1 << length_bits) - 1
        }

        Self {
            offset_bits,
            length_bits,
            max_offset,
            max_length,
            window: Vec::new(),
            lab_start_index: 0,
            tokens: Vec::new(),
        }
    }

    pub fn encode(&mut self, input_data: &[u8]) -> Vec<u8> {
        self.tokens.clear();
        let mut writer = BitWriter::new();

        self.write_header(&mut writer);

        let bytes_read = self.initialize_window(input_data);
        let mut input_position = bytes_read;

        while !self.get_lookahead_buffer().is_empty() {
            let (best_match_offset, best_match_length) = self.find_longest_match();
            let next_char = self.get_lookahead_buffer()[best_match_length];

            self.tokens
                .push(Token::new(best_match_offset, best_match_length, next_char));

            self.emit_token(&mut writer, best_match_offset, best_match_length, next_char);
            self.slide_window(best_match_length + 1, input_data, &mut input_position);
        }

        writer.write_n_bits(7, 0);
        writer.finish()
    }

    fn write_header(&self, writer: &mut BitWriter) {
        writer.write_n_bits(4, self.offset_bits as u32);
        writer.write_n_bits(3, self.length_bits as u32);
    }

    fn initialize_window(&mut self, data: &[u8]) -> usize {
        let bytes_to_read = min(self.max_length, data.len());
        self.lab_start_index = 0;

        self.window.clear();
        if bytes_to_read > 0 {
            self.window.extend_from_slice(&data[..bytes_to_read]);
        }

        bytes_to_read
    }

    fn find_longest_match(&self) -> (usize, usize) {
        let search_buffer = self.get_search_buffer();
        let look_ahead_buffer = self.get_lookahead_buffer();

        if search_buffer.is_empty() || look_ahead_buffer.is_empty() {
            return (0, 0);
        }

        let mut best_match_offset = 0;
        let mut best_match_length = 0;

        let max_match_length = min(self.max_length, look_ahead_buffer.len().saturating_sub(1));

        for search_buffer_pos in 0..search_buffer.len() {
            let mut match_length = 0;

            for i in 0..max_match_length {
                let window_pos = search_buffer_pos + i;

                if self.window.len() <= window_pos {
                    break;
                }

                if self.window[window_pos] != look_ahead_buffer[i] {
                    break;
                }

                match_length += 1;
            }

            if best_match_length < match_length {
                best_match_length = match_length;
                best_match_offset = self.lab_start_index - search_buffer_pos;
            }
        }

        (best_match_offset, best_match_length)
    }

    fn emit_token(&self, writer: &mut BitWriter, offset: usize, length: usize, next_char: u8) {
        writer.write_n_bits(self.offset_bits as u32, offset as u32);
        writer.write_n_bits(self.length_bits as u32, length as u32);
        writer.write_n_bits(8, next_char as u32);
    }

    fn slide_window(&mut self, n: usize, input: &[u8], input_position: &mut usize) {
        self.lab_start_index += n;

        let bytes_to_read = min(n, input.len() - *input_position);

        if bytes_to_read > 0 {
            self.window
                .extend_from_slice(&input[*input_position..*input_position + bytes_to_read]);
            *input_position += bytes_to_read;
        }

        if self.lab_start_index > self.max_offset {
            let excess = self.lab_start_index - self.max_offset;

            self.window.drain(0..excess);
            self.lab_start_index -= excess;
        }
    }

    pub fn decode(&mut self, encoded_data: &[u8]) -> Vec<u8> {
        let mut reader = BitReader::new(encoded_data);
        let mut output: Vec<u8> = Vec::new();

        let (offset_bits, length_bits) = match Self::read_header(&mut reader) {
            (offset, length) => (offset, length),
        };

        self.offset_bits = offset_bits;
        self.length_bits = length_bits;
        self.max_offset = (1 << self.offset_bits) - 1;
        self.max_length = (1 << self.length_bits) - 1;

        while let Some(token) = self.read_token(&mut reader) {
            self.decode_token(&token, &mut output);
        }

        output
    }

    fn read_header(reader: &mut BitReader) -> (u8, u8) {
        let offset_bits = match reader.read_n_bits(4) {
            Some(value) => value as u8,
            None => 0,
        };
        let length_bits = match reader.read_n_bits(3) {
            Some(value) => value as u8,
            None => 0,
        };
        (offset_bits, length_bits)
    }

    fn read_token(&self, reader: &mut BitReader) -> Option<Token> {
        let offset_raw = match reader.read_n_bits(self.offset_bits as u32) {
            Some(v) => v,
            None => return None,
        };

        let length_raw = match reader.read_n_bits(self.length_bits as u32) {
            Some(v) => v,
            None => return None,
        };

        let ch_raw = match reader.read_n_bits(8) {
            Some(v) => v,
            None => return None,
        };

        Some(Token::new(
            offset_raw as usize,
            length_raw as usize,
            ch_raw as u8,
        ))
    }

    fn decode_token(&self, token: &Token, output: &mut Vec<u8>) {
        let match_start_position = output.len() - token.offset;

        if token.offset == 0 || token.match_length == 0 {
            output.push(token.next_char);
            return;
        }

        for i in 0..token.match_length {
            let index = match_start_position + i;
            let symbol = output[index];
            output.push(symbol);
        }

        output.push(token.next_char);
    }

    fn get_search_buffer(&self) -> &[u8] {
        &self.window[0..self.lab_start_index]
    }

    fn get_lookahead_buffer(&self) -> &[u8] {
        &self.window[self.lab_start_index..]
    }

    pub fn calculate_compression_ratio(original_size: usize, compressed_size: usize) -> f64 {
        if original_size == 0 {
            return 0.0;
        }
        ((original_size as f64 - compressed_size as f64) / original_size as f64) * 100.0
    }

    pub fn get_tokens(&self) -> Vec<Token> {
        self.tokens.clone()
    }
}
