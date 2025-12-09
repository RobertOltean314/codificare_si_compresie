use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CompressRequest {
    pub filename: String,
    pub data: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DecompressRequest {
    pub filename: String,
    pub data: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EncodingOptions {
    #[serde(default)]
    pub auto_update_index: bool,
    pub manual_index_bits: Option<u8>,
    #[serde(default)]
    pub show_emitted_codes: bool,
}

impl EncodingOptions {
    pub fn validate(&self) -> Result<(), String> {
        if !self.auto_update_index {
            match self.manual_index_bits {
                Some(bits) if bits >= 9 && bits <= 15 => Ok(()),
                Some(bits) => Err(format!(
                    "Manual index bits must be between 9-15, got {}",
                    bits
                )),
                None => {
                    Err("Manual index bits required when auto_update_index is false".to_string())
                }
            }
        } else {
            Ok(())
        }
    }

    pub fn get_initial_bit_width(&self) -> u32 {
        if self.auto_update_index {
            9
        } else {
            self.manual_index_bits.unwrap_or(9) as u32
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct DecodingOptions {
    #[serde(default)]
    pub show_codes: bool,
}

#[derive(Debug, Serialize)]
pub struct EncodeResponse {
    pub success: bool,
    pub message: String,
    pub filename: String,
    pub original_size: usize,
    pub compressed_size: usize,
    pub header_size: usize,
    pub compressed_data_size: usize,
    pub compression_ratio: f64,
    pub space_saved: usize,
    pub percentage_saved: f64,
    pub codes: Option<Vec<usize>>,
    pub file_data: String,
}

#[derive(Debug, Serialize)]
pub struct DecodeResponse {
    pub success: bool,
    pub message: String,
    pub filename: String,
    pub original_size: usize,
    pub decompressed_size: usize,
    pub codes: Option<Vec<(String, String)>>,
    pub file_data: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
}
