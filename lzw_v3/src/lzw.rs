use crate::bit_operations::{BitReader, BitWriter};
use std::collections::HashMap;

pub struct LZW {
    pub initial_dictionary: HashMap<Vec<u8>, usize>,
}

impl LZW {
    pub fn new() -> Self {
        let initial_dictionary = Self::generate_initial_dictionary();
        LZW { initial_dictionary }
    }

    fn generate_initial_dictionary() -> HashMap<Vec<u8>, usize> {
        let mut dictionary = HashMap::new();

        for byte in 0..=255u8 {
            dictionary.insert(vec![byte], byte as usize);
        }

        dictionary
    }

    fn generate_inverse_dictionary() -> HashMap<usize, Vec<u8>> {
        let mut dictionary = HashMap::new();

        for byte in 0..=255u8 {
            dictionary.insert(byte as usize, vec![byte]);
        }

        dictionary
    }

    pub fn write_header(writer: &mut BitWriter, auto_update_flag: bool, manual_bits: Option<u8>) {
        if auto_update_flag {
            writer.write_bit(true);
        } else {
            writer.write_bit(false);

            let index_bits = manual_bits.unwrap_or(9);
            let encoded_size = (index_bits - 9) as u32;
            writer.write_n_bits(3, encoded_size);
            writer.write_bit(false);
        }
    }

    fn lzw_algorithm(
        &mut self,
        file_data: &[u8],
        auto_update: bool,
        initial_bit_length: u32,
        writer: &mut BitWriter,
    ) -> Vec<usize> {
        let mut next_available_code = 256;
        let mut current_bit_length = initial_bit_length;
        let max_bit_width = match auto_update {
            true => 15,
            false => initial_bit_length,
        };

        let mut max_code = 1 << current_bit_length;
        let mut emitted_codes = Vec::new();
        let mut current_pattern: Vec<u8> = Vec::new();

        for &character in file_data {
            let mut extended_pattern = current_pattern.clone();
            extended_pattern.push(character);

            if self.initial_dictionary.contains_key(&extended_pattern) {
                current_pattern = extended_pattern;
            } else {
                self.emit_code(
                    &current_pattern,
                    current_bit_length,
                    writer,
                    &mut emitted_codes,
                );

                if next_available_code >= max_code {
                    self.handle_compression_dictionary_full(
                        &mut current_bit_length,
                        max_bit_width,
                        &mut max_code,
                        &mut next_available_code,
                    );
                }
                self.add_to_dictionary(extended_pattern, &mut next_available_code, max_code);
                current_pattern = vec![character];
            }
        }

        if !current_pattern.is_empty() {
            self.emit_code(
                &current_pattern,
                current_bit_length,
                writer,
                &mut emitted_codes,
            );
        }

        emitted_codes
    }

    fn emit_code(
        &self,
        pattern: &[u8],
        current_bit_length: u32,
        writer: &mut BitWriter,
        emitted_codes: &mut Vec<usize>,
    ) {
        if let Some(&code) = self.initial_dictionary.get(pattern) {
            writer.write_n_bits(current_bit_length, code as u32);
            emitted_codes.push(code);
        }
    }

    fn handle_compression_dictionary_full(
        &mut self,
        current_bit_width: &mut u32,
        max_bit_width: u32,
        max_code: &mut usize,
        next_available_code: &mut usize,
    ) {
        if *current_bit_width < max_bit_width {
            *current_bit_width += 1;
            *max_code = 1 << *current_bit_width;
        } else {
            self.initial_dictionary.clear();
            for byte in 0..=255u8 {
                self.initial_dictionary.insert(vec![byte], byte as usize);
            }
            *next_available_code = 256;
        }
    }

    fn handle_decompression_dictionary_full(
        dictionary: &mut HashMap<usize, Vec<u8>>,
        current_bit_width: &mut u32,
        max_bit_width: u32,
        max_code: &mut usize,
        next_available_code: &mut usize,
        use_empty_mode: bool,
    ) {
        if *current_bit_width < max_bit_width {
            *current_bit_width += 1;
            *max_code = 1 << *current_bit_width;
        } else if use_empty_mode {
            dictionary.clear();
            for byte in 0..=255u8 {
                dictionary.insert(byte as usize, vec![byte]);
            }
            *next_available_code = 256;
        }
    }

    fn add_to_dictionary(
        &mut self,
        pattern: Vec<u8>,
        next_available_code: &mut usize,
        max_code: usize,
    ) {
        if *next_available_code < max_code {
            self.initial_dictionary
                .insert(pattern, *next_available_code);
            *next_available_code += 1;
        }
    }

    pub fn compress(
        &mut self,
        file_data: &[u8],
        auto_update: bool,
        initial_bit_width: u32,
        manual_bits: Option<u8>,
    ) -> (Vec<u8>, Vec<usize>) {
        let mut writer = BitWriter::new();

        Self::write_header(&mut writer, auto_update, manual_bits);

        let emitted_codes =
            self.lzw_algorithm(file_data, auto_update, initial_bit_width, &mut writer);

        writer.write_n_bits(7, 0b1111111);
        writer.flush();

        (writer.into_bytes(), emitted_codes)
    }

    pub fn decompress(&self, compressed_data: &[u8]) -> (Vec<u8>, Vec<usize>) {
        let mut output = Vec::new();
        let mut decoded_codes = Vec::new();
        let mut reader = BitReader::new(compressed_data);

        let (mut current_bit_width, max_bit_width, use_empty_mode) =
            match Self::read_header(&mut reader) {
                Some(header) => header,
                None => return (output, decoded_codes),
            };

        let mut dictionary = Self::generate_inverse_dictionary();
        let mut max_code = 1 << current_bit_width;
        let mut next_available_code = 256;

        let first_code = match reader.read_n_bits(current_bit_width) {
            Some(code) => code as usize,
            None => return (output, decoded_codes),
        };

        if let Some(pattern) = dictionary.get(&first_code) {
            output.extend_from_slice(pattern);
            decoded_codes.push(first_code);
        }

        let mut previous_code = first_code;

        while let Some(current_code_raw) = reader.read_n_bits(current_bit_width) {
            let current_code = current_code_raw as usize;
            decoded_codes.push(current_code);

            let decoded_pattern = Self::decode_pattern(
                &dictionary,
                current_code,
                previous_code,
                next_available_code,
            );

            output.extend_from_slice(&decoded_pattern);
            if !decoded_pattern.is_empty() && next_available_code < max_code {
                Self::add_decompression_entry(
                    &mut dictionary,
                    previous_code,
                    decoded_pattern[0],
                    next_available_code,
                );
                next_available_code += 1;
            }

            if next_available_code >= max_code {
                Self::handle_decompression_dictionary_full(
                    &mut dictionary,
                    &mut current_bit_width,
                    max_bit_width,
                    &mut max_code,
                    &mut next_available_code,
                    use_empty_mode,
                );
            }

            previous_code = current_code;
        }

        (output, decoded_codes)
    }

    fn read_header(reader: &mut BitReader) -> Option<(u32, u32, bool)> {
        let auto_update = reader.read_bit()?;

        let (current_bit_width, max_bit_width, use_empty_mode) = if auto_update {
            (9, 15, false)
        } else {
            let size_bits = reader.read_n_bits(3)?;
            let mode_bit = reader.read_bit()?;
            let bit_width = (size_bits + 9) as u32;
            let empty_mode = !mode_bit;
            (bit_width, bit_width, empty_mode)
        };

        Some((current_bit_width, max_bit_width, use_empty_mode))
    }

    fn decode_pattern(
        dictionary: &HashMap<usize, Vec<u8>>,
        current_code: usize,
        previous_code: usize,
        next_available_code: usize,
    ) -> Vec<u8> {
        if dictionary.contains_key(&current_code) {
            dictionary.get(&current_code).unwrap().clone()
        } else if current_code == next_available_code {
            let mut pattern = dictionary.get(&previous_code).unwrap().clone();
            pattern.push(pattern[0]);
            pattern
        } else {
            Vec::new()
        }
    }

    fn add_decompression_entry(
        dictionary: &mut HashMap<usize, Vec<u8>>,
        previous_code: usize,
        first_char: u8,
        next_available_code: usize,
    ) {
        let mut new_entry = dictionary.get(&previous_code).unwrap().clone();
        new_entry.push(first_char);
        dictionary.insert(next_available_code, new_entry);
    }
}
