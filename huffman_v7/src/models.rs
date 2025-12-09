use serde::{Deserialize, Serialize};

#[derive(Serialize)]
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
    pub codes: Option<Vec<(String, String)>>,
    pub file_data: String,
}

#[derive(Serialize)]
pub struct DecodeResponse {
    pub success: bool,
    pub message: String,
    pub filename: String,
    pub original_size: usize,
    pub decompressed_size: usize,
    pub codes: Option<Vec<(String, String)>>,
    pub file_data: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
}

#[derive(Deserialize)]
pub struct EncodingOptions {
    #[serde(default)]
    pub two_bytes: bool,
    #[serde(default)]
    pub show_codes: bool,
}

#[derive(Deserialize)]
pub struct DecodingOptions {
    #[serde(default)]
    pub show_codes: bool,
}
