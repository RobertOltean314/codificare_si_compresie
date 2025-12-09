use actix_multipart::Multipart;
use actix_web::{HttpResponse, Result, web};
use base64::{Engine as _, engine::general_purpose};
use futures_util::StreamExt;

use crate::{
    lzw::LZW,
    models::{DecodeResponse, DecodingOptions, EncodeResponse, EncodingOptions, ErrorResponse},
};

pub async fn encode_file(
    mut payload: Multipart,
    query: web::Query<EncodingOptions>,
) -> Result<HttpResponse> {
    let mut file_data = Vec::new();
    let mut filename = String::from("unknown");

    while let Some(item) = payload.next().await {
        let mut field = item?;

        if let Some(content_disposition) = field.content_disposition() {
            if let Some(name) = content_disposition.get_name() {
                if name == "file" {
                    if let Some(fname) = content_disposition.get_filename() {
                        filename = fname.to_string();
                    }

                    while let Some(chunk) = field.next().await {
                        let data = chunk?;
                        file_data.extend_from_slice(&data);
                    }
                }
            }
        }
    }

    if file_data.is_empty() {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: "No file data received".to_string(),
        }));
    }

    if let Err(err) = query.validate() {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: err,
        }));
    }

    let mut lzw = LZW::new();
    let auto_update = query.auto_update_index;
    let initial_bit_width = query.get_initial_bit_width();
    let manual_bits = query.manual_index_bits;

    let (compressed_bytes, emitted_codes) =
        lzw.compress(&file_data, auto_update, initial_bit_width, manual_bits);

    let original_size = file_data.len();
    let compressed_size = compressed_bytes.len();

    let header_size = if auto_update { 1 } else { 5 };

    let compressed_data_size = compressed_size.saturating_sub(1);

    let compression_ratio = if compressed_size > 0 {
        original_size as f64 / compressed_size as f64
    } else {
        0.0
    };

    let space_saved = original_size.saturating_sub(compressed_size);
    let percentage_saved = if original_size > 0 {
        (space_saved as f64 / original_size as f64) * 100.0
    } else {
        0.0
    };

    let bits = manual_bits.unwrap_or(9);
    let output_filename = format!("{}.e{}.LZW", filename, bits);

    let file_data_base64 = general_purpose::STANDARD.encode(&compressed_bytes);

    Ok(HttpResponse::Ok().json(EncodeResponse {
        success: true,
        message: "File encoded successfully".to_string(),
        filename: output_filename,
        original_size,
        compressed_size,
        header_size,
        compressed_data_size,
        compression_ratio,
        space_saved,
        percentage_saved,
        codes: if query.show_emitted_codes {
            Some(emitted_codes)
        } else {
            None
        },
        file_data: file_data_base64,
    }))
}

pub async fn decode_file(
    mut payload: Multipart,
    query: web::Query<DecodingOptions>,
) -> Result<HttpResponse> {
    let mut file_data = Vec::new();
    let mut filename = String::from("unknown");

    while let Some(item) = payload.next().await {
        let mut field = item?;

        if let Some(content_disposition) = field.content_disposition() {
            if let Some(name) = content_disposition.get_name() {
                if name == "file" {
                    if let Some(fname) = content_disposition.get_filename() {
                        filename = fname.to_string();
                    }

                    while let Some(chunk) = field.next().await {
                        let data = chunk?;
                        file_data.extend_from_slice(&data);
                    }
                }
            }
        }
    }

    if file_data.is_empty() {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: "No file data received".to_string(),
        }));
    }

    if !filename.to_lowercase().ends_with(".lzw") {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: "File must have .LZW extension".to_string(),
        }));
    }

    let lzw = LZW::new();
    let (decompressed_bytes, decoded_codes) = lzw.decompress(&file_data);

    let original_size = file_data.len();
    let decompressed_size = decompressed_bytes.len();

    let output_filename = if let Some(base) = filename.strip_suffix(".LZW") {
        let parts: Vec<&str> = base.split('.').collect();
        if parts.len() >= 2 {
            let ext = parts[parts.len() - 2];
            format!("{}.{}", filename, ext)
        } else {
            format!("{}.txt", filename)
        }
    } else {
        format!("{}.txt", filename)
    };

    let file_data_base64 = general_purpose::STANDARD.encode(&decompressed_bytes);

    let codes_display = if query.show_codes {
        Some(
            decoded_codes
                .iter()
                .enumerate()
                .map(|(i, code)| (format!("{}", i), format!("{}", code)))
                .collect(),
        )
    } else {
        None
    };

    Ok(HttpResponse::Ok().json(DecodeResponse {
        success: true,
        message: "File decoded successfully".to_string(),
        filename: output_filename,
        original_size,
        decompressed_size,
        codes: codes_display,
        file_data: file_data_base64,
    }))
}

pub async fn index() -> Result<actix_files::NamedFile> {
    Ok(actix_files::NamedFile::open("static/index.html")?)
}
