use crate::bit_operations::{BitReader, BitWriter};
use crate::tree::{Node, Symbol};
use std::collections::{BinaryHeap, HashMap};

#[derive(Debug, Clone)]
pub struct CompressionStats {
    pub original_size: usize,
    pub compressed_size: usize,
    pub header_size: usize,
    pub compressed_data_size: usize,
    pub compression_ratio: f64,
    pub space_saved: usize,
    pub percentage_saved: f64,
}

#[derive(Clone)]
pub struct Huffman {
    one_byte_counters: HashMap<u8, u32>,
    two_bytes_counters: HashMap<u16, u32>,
    symbol: Symbol,
    last_codes: Option<HashMap<Symbol, Vec<bool>>>,
    header_size: usize,
}

impl Huffman {
    pub fn new(symbol: Symbol) -> Self {
        Self {
            one_byte_counters: HashMap::new(),
            two_bytes_counters: HashMap::new(),
            symbol,
            last_codes: None,
            header_size: 0,
        }
    }

    pub fn count_frequencies(&mut self, file_data: &[u8]) {
        let mut reader: BitReader = BitReader::new(&file_data);

        match self.symbol {
            Symbol::OneByte(_) => {
                while let Some(byte) = reader.read_n_bits(8) {
                    *self.one_byte_counters.entry(byte as u8).or_insert(0) += 1;
                }
            }
            Symbol::TwoBytes(_) => {
                while let Some(two_bytes) = reader.read_n_bits(16) {
                    *self.two_bytes_counters.entry(two_bytes as u16).or_insert(0) += 1;
                }
            }
        }
    }

    pub fn build_huffman_tree(&self) -> Node {
        let mut heap = BinaryHeap::new();

        match self.symbol {
            Symbol::OneByte(_) => {
                for (&byte, &frequency) in self.one_byte_counters.iter() {
                    if frequency > 0 {
                        let symbol = Symbol::OneByte(byte);
                        let leaf = Node::new(symbol, frequency);
                        heap.push(leaf);
                    }
                }
            }
            Symbol::TwoBytes(_) => {
                for (&symbol_value, &frequency) in self.two_bytes_counters.iter() {
                    if frequency > 0 {
                        let symbol = Symbol::TwoBytes(symbol_value);
                        let leaf = Node::new(symbol, frequency);
                        heap.push(leaf);
                    }
                }
            }
        }

        while heap.len() > 1 {
            let left = heap.pop().unwrap();
            let right = heap.pop().unwrap();
            let parent = Node::new_parent(left, right);
            heap.push(parent);
        }

        heap.pop()
            .expect("Huffman tree should have at least one node")
    }

    fn generate_huffman_codes(&self, root: &Node) -> HashMap<Symbol, Vec<bool>> {
        let mut codes = HashMap::new();
        let mut current_code = Vec::new();
        self.generate_codes_recursive(root, &mut current_code, &mut codes);
        codes
    }

    fn generate_codes_recursive(
        &self,
        node: &Node,
        current_code: &mut Vec<bool>,
        codes: &mut HashMap<Symbol, Vec<bool>>,
    ) {
        if node.is_leaf() {
            codes.insert(node.symbol, current_code.clone());
            return;
        }

        if let Some(ref left) = node.left {
            current_code.push(false);
            println!("Current code: {:?}", current_code);
            self.generate_codes_recursive(left, current_code, codes);
            current_code.pop();
        }

        if let Some(ref right) = node.right {
            current_code.push(true);
            println!("Current code: {:?}", current_code);
            self.generate_codes_recursive(right, current_code, codes);
            current_code.pop();
        }
    }

    fn write_codes(
        &self,
        reader: &mut BitReader,
        writer: &mut BitWriter,
        codes: &HashMap<Symbol, Vec<bool>>,
    ) {
        match self.symbol {
            Symbol::OneByte(_) => {
                while let Some(byte) = reader.read_n_bits(8) {
                    let symbol = Symbol::OneByte(byte as u8);
                    if let Some(code) = codes.get(&symbol) {
                        for &bit in code {
                            writer.write_bit(bit);
                        }
                    }
                }
            }
            Symbol::TwoBytes(_) => {
                while let Some(symbol_value) = reader.read_n_bits(16) {
                    let symbol = Symbol::TwoBytes(symbol_value as u16);
                    if let Some(code) = codes.get(&symbol) {
                        for &bit in code {
                            writer.write_bit(bit);
                        }
                    }
                }
            }
        }
    }

    fn write_frequencies_in_header(&self, writer: &mut BitWriter) {
        match self.symbol {
            Symbol::OneByte(_) => self.write_frequency_header_on_one_byte(writer),
            Symbol::TwoBytes(_) => self.write_frequency_header_on_two_bytes(writer),
        }
    }

    fn write_frequency_header_on_one_byte(&self, writer: &mut BitWriter) {
        for symbol in 0u8..=255u8 {
            let frequency = self.one_byte_counters.get(&symbol).copied().unwrap_or(0);
            let encoding = self.get_frequency_encoding(frequency);

            writer.write_bit((encoding & 0b10) != 0);
            writer.write_bit((encoding & 0b01) != 0);
        }
    }

    fn write_frequency_header_on_two_bytes(&self, writer: &mut BitWriter) {
        for symbol in 0..65536u32 {
            let exists = self.two_bytes_counters.contains_key(&(symbol as u16));
            writer.write_bit(exists);
        }

        for symbol in 0..65536u32 {
            if let Some(&frequency) = self.two_bytes_counters.get(&(symbol as u16)) {
                if frequency <= 255 {
                    writer.write_bit(false);
                } else {
                    writer.write_bit(true);
                }
            }
        }
    }

    fn write_frequency_values_in_header(&self, writer: &mut BitWriter) {
        match self.symbol {
            Symbol::OneByte(_) => self.write_frequency_value_on_one_byte(writer),
            Symbol::TwoBytes(_) => self.write_frequency_values_on_two_bytes(writer),
        }
    }

    fn write_frequency_value_on_one_byte(&self, writer: &mut BitWriter) {
        for symbol in 0u8..=255u8 {
            let frequency = self.one_byte_counters.get(&symbol).copied().unwrap_or(0);
            let encoding = self.get_frequency_encoding(frequency);

            match encoding {
                0b00 => {}
                0b01 => {
                    writer.write_n_bits(frequency, 8);
                }
                0b10 => {
                    writer.write_n_bits(frequency, 16);
                }
                0b11 => {
                    panic!("Frequency {} higher then 65,535", frequency)
                }
                _ => unreachable!(),
            }
        }
    }

    fn write_frequency_values_on_two_bytes(&self, writer: &mut BitWriter) {
        for symbol in 0..65536u32 {
            if let Some(&frequency) = self.two_bytes_counters.get(&(symbol as u16)) {
                if frequency <= 255 {
                    writer.write_n_bits(frequency, 8);
                } else if frequency <= 65535 {
                    writer.write_n_bits(frequency, 16);
                } else {
                    writer.write_n_bits(frequency, 32);
                }
            }
        }
    }

    fn get_frequency_encoding(&self, frequency: u32) -> u8 {
        if frequency == 0 {
            0b00
        } else if frequency <= 255 {
            0b01
        } else if frequency <= 65535 {
            0b10
        } else {
            0b11
        }
    }

    pub fn compress(&mut self, file_data: &[u8]) -> Vec<u8> {
        let mut writer = BitWriter::new();
        let mut reader = BitReader::new(file_data);

        self.count_frequencies(file_data);

        match self.symbol {
            Symbol::OneByte(_) => writer.write_bit(false),
            Symbol::TwoBytes(_) => writer.write_bit(true),
        }

        self.write_frequencies_in_header(&mut writer);
        self.write_frequency_values_in_header(&mut writer);

        self.header_size = writer.data.len() + if writer.bit_position > 0 { 1 } else { 0 };

        let tree = self.build_huffman_tree();
        let codes = self.generate_huffman_codes(&tree);

        self.last_codes = Some(codes.clone());

        self.write_codes(&mut reader, &mut writer, &codes);

        writer.finish()
    }

    pub fn decompress(&mut self, file_data: &[u8]) -> Vec<u8> {
        let mut reader = BitReader::new(file_data);
        let mut writer = BitWriter::new();

        let mode_bit = reader.read_bit().unwrap_or(false);
        self.symbol = if mode_bit {
            Symbol::TwoBytes(0)
        } else {
            Symbol::OneByte(0)
        };

        self.read_frequency_header(&mut reader);

        let tree = self.build_huffman_tree();

        let total_symbols = self.calculate_total_symbols();

        let mut decoded_symbols = 0;
        let mut current_node = &tree;

        while decoded_symbols < total_symbols {
            let bit = match reader.read_bit() {
                Some(b) => b,
                None => break,
            };

            if bit == false {
                current_node = current_node.left.as_ref().unwrap();
            } else {
                current_node = current_node.right.as_ref().unwrap();
            }

            if current_node.is_leaf() {
                match current_node.symbol {
                    Symbol::OneByte(byte) => {
                        writer.write_n_bits(byte as u32, 8);
                    }
                    Symbol::TwoBytes(symbol_val) => {
                        writer.write_n_bits(symbol_val as u32, 16);
                    }
                }
                decoded_symbols += 1;
                current_node = &tree;
            }
        }

        writer.finish()
    }

    fn read_frequency_header(&mut self, reader: &mut BitReader) {
        match self.symbol {
            Symbol::OneByte(_) => self.read_frequency_header_on_one_byte(reader),
            Symbol::TwoBytes(_) => self.read_frequency_header_on_two_bytes(reader),
        }
    }

    fn read_frequency_header_on_one_byte(&mut self, reader: &mut BitReader) {
        self.one_byte_counters.clear();

        let mut encodings = [0u8; 256];
        for symbol in 0u8..=255u8 {
            let high_bit = reader.read_bit().unwrap_or(false);
            let low_bit = reader.read_bit().unwrap_or(false);

            encodings[symbol as usize] =
                if high_bit { 0b10 } else { 0b00 } | if low_bit { 0b01 } else { 0b00 };
        }

        for symbol in 0u8..=255u8 {
            let encoding = encodings[symbol as usize];

            let frequency = match encoding {
                0b00 => 0,
                0b01 => reader.read_n_bits(8).unwrap_or(0),
                0b10 => reader.read_n_bits(16).unwrap_or(0),
                0b11 => reader.read_n_bits(32).unwrap_or(0),
                _ => unreachable!(),
            };

            if frequency > 0 {
                self.one_byte_counters.insert(symbol, frequency);
            }
        }
    }

    fn read_frequency_header_on_two_bytes(&mut self, reader: &mut BitReader) {
        self.two_bytes_counters.clear();

        let mut existing_symbols = Vec::new();
        for symbol in 0..65536u32 {
            let exists = reader.read_bit().unwrap_or(false);
            if exists {
                existing_symbols.push(symbol as u16);
            }
        }
        for &symbol in &existing_symbols {
            let size_bit = reader.read_bit().unwrap_or(false);

            let frequency = if size_bit {
                reader.read_n_bits(16).unwrap_or(0)
            } else {
                reader.read_n_bits(8).unwrap_or(0)
            };

            if frequency > 0 {
                self.two_bytes_counters.insert(symbol, frequency);
            }
        }
    }

    fn calculate_total_symbols(&self) -> u32 {
        match self.symbol {
            Symbol::OneByte(_) => self.one_byte_counters.values().sum(),
            Symbol::TwoBytes(_) => self.two_bytes_counters.values().sum(),
        }
    }

    pub fn get_codes(&self) -> Option<Vec<(String, String)>> {
        self.last_codes.as_ref().map(|codes| {
            let mut result: Vec<(String, String)> = codes
                .iter()
                .map(|(symbol, code)| {
                    let symbol_str = match symbol {
                        Symbol::OneByte(byte) => {
                            if *byte >= 32 && *byte <= 126 {
                                format!("{} (0x{:02X})", *byte as char, byte)
                            } else {
                                format!("byte 0x{:02X}", byte)
                            }
                        }
                        Symbol::TwoBytes(val) => {
                            format!("bytes 0x{:04X}", val)
                        }
                    };

                    let code_str = code
                        .iter()
                        .map(|&bit| if bit { '1' } else { '0' })
                        .collect::<String>();

                    (symbol_str, code_str)
                })
                .collect();

            result.sort_by(|a, b| a.0.cmp(&b.0));
            result
        })
    }

    pub fn get_compression_stats(
        &self,
        original_size: usize,
        compressed_size: usize,
    ) -> CompressionStats {
        let compressed_data_size = compressed_size.saturating_sub(self.header_size);

        CompressionStats {
            original_size,
            compressed_size,
            header_size: self.header_size,
            compressed_data_size,
            compression_ratio: if original_size > 0 {
                ((original_size.saturating_sub(compressed_size)) as f64 / original_size as f64)
                    * 100.0
            } else {
                0.0
            },
            space_saved: original_size.saturating_sub(compressed_data_size),
            percentage_saved: if original_size > 0 {
                ((original_size.saturating_sub(compressed_data_size)) as f64 / original_size as f64)
                    * 100.0
            } else {
                0.0
            },
        }
    }
}
