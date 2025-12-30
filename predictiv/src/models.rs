use serde::{Deserialize, Serialize};

pub type Predictor = fn(a: i32, b: i32, c: i32) -> i32;

pub struct MyCustomImage {
    pub height: usize,
    pub width: usize,
    pub data: Vec<Vec<i32>>,
}

#[derive(Debug)]
pub struct Histogram(pub [u32; 256]);

impl Serialize for Histogram {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}
#[derive(Debug, Deserialize)]
pub struct DecodeRequest {
    pub file_name: String,
    pub file_data: Vec<u8>,
}

#[derive(Debug, Serialize)]
pub struct DecodeResponse {
    pub decoded_filename: String,
    pub decoded_image: Vec<Vec<i32>>,
    pub predicted_type: u8,
    pub decoded_bmp_data: Vec<u8>,
}

#[derive(Debug, Serialize)]
pub struct EncodeResponse {
    pub encoded_filename: String,
    pub original_image: Vec<Vec<i32>>,
    pub error_matrix: Vec<Vec<i32>>,
    pub prediction_type: u8,
    pub encoded_data: Vec<u8>,
    pub original_histogram: Histogram,
    pub error_histogram: Histogram,
}

#[derive(Debug, Deserialize)]
pub struct EncodeRequest {
    pub file_name: String,
    pub file_data: Vec<u8>,
    pub prediction_number: u8,
}
