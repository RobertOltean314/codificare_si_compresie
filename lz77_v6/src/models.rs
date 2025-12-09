use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub offset: usize,
    pub match_length: usize,
    pub next_char: u8,
}

impl Token {
    pub fn new(offset: usize, length: usize, next_char: u8) -> Self {
        Self {
            offset,
            match_length: length,
            next_char,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct EncodeRequest {
    pub filename: String,
    pub file_data: Vec<u8>,
    pub offset_bits: u8,
    pub length_bits: u8,
}

#[derive(Debug, Serialize)]
pub struct EncodeResponse {
    pub encoded_filename: String,
    pub encoded_data: Vec<u8>,
    pub original_size: usize,
    pub compressed_size: usize,
    pub compression_ratio: f64,
    pub tokens: Option<Vec<Token>>,
}

#[derive(Debug, Deserialize)]
pub struct DecodeRequest {
    pub filename: String,
    pub file_data: Vec<u8>,
}

#[derive(Debug, Serialize)]
pub struct DecodeResponse {
    pub decoded_filename: String,
    pub decoded_data: Vec<u8>,
    pub original_compressed_size: usize,
    pub decompressed_size: usize,
}

#[derive(Debug, Serialize)]
pub struct CompressionStats {
    pub offset_bits: u8,
    pub length_bits: u8,
    pub original_size: usize,
    pub compressed_size: usize,
    pub compression_ratio: f64,
    pub encoding_time_ms: u128,
}
