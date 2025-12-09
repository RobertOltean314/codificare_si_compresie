use actix_web::{HttpResponse, Result, web};

use crate::lz77::LZ77;
use crate::models::{DecodeRequest, DecodeResponse, EncodeRequest, EncodeResponse};

pub async fn encode_file(req: web::Json<EncodeRequest>) -> Result<HttpResponse> {
    let mut lz = LZ77::new(Some(req.offset_bits), Some(req.length_bits));
    let encoded_bytes = lz.encode(&req.file_data);

    let original_size = req.file_data.len();
    let compressed_size = encoded_bytes.len();
    let compression_ratio = LZ77::calculate_compression_ratio(original_size, compressed_size);

    let encoded_filename = format!(
        "{}.o{}l{}.lz77",
        req.filename, req.offset_bits, req.length_bits
    );

    let tokens = lz.get_tokens();

    let response = EncodeResponse {
        encoded_filename,
        encoded_data: encoded_bytes,
        original_size,
        compressed_size,
        compression_ratio,
        tokens: Some(tokens),
    };

    Ok(HttpResponse::Ok().json(response))
}

pub async fn decode_file(req: web::Json<DecodeRequest>) -> Result<HttpResponse> {
    let mut lz = LZ77::new(None, None);
    let decoded_bytes = lz.decode(&req.file_data);

    let original_compressed_size = req.file_data.len();
    let decompressed_size = decoded_bytes.len();

    let decoded_filename = extract_original_filename(&req.filename);

    let response = DecodeResponse {
        decoded_filename,
        decoded_data: decoded_bytes,
        original_compressed_size,
        decompressed_size,
    };

    Ok(HttpResponse::Ok().json(response))
}

fn extract_original_filename(encoded_filename: &str) -> String {
    if let Some(pos) = encoded_filename.rfind(".o") {
        if encoded_filename.ends_with(".lz77") {
            return encoded_filename[..pos].to_string();
        }
    }

    encoded_filename
        .strip_suffix(".lz77")
        .unwrap_or(encoded_filename)
        .to_string()
}

pub async fn index() -> Result<actix_files::NamedFile> {
    Ok(actix_files::NamedFile::open("static/index.html")?)
}
