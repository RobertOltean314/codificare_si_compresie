use crate::models::{DecodeResponse, DecodingOptions, EncodeResponse, ErrorResponse};
use crate::tree::Symbol;
use crate::{huffman::Huffman, models::EncodingOptions};
use actix_files;
use actix_multipart::Multipart;
use actix_web::{web, HttpResponse, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::Local;
use futures_util::StreamExt;

pub async fn encode_file(
    mut payload: Multipart,
    query: web::Query<EncodingOptions>,
) -> Result<HttpResponse> {
    let mut file_data = Vec::new();
    let mut filename = String::from("unknown");
    let use_two_bytes = query.two_bytes;
    let show_codes = query.show_codes;

    while let Some(item) = payload.next().await {
        let mut field = item?;

        let content_disposition = field.content_disposition();
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

    if file_data.is_empty() {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: "No file data received".to_string(),
        }));
    }

    let symbol = if use_two_bytes {
        Symbol::TwoBytes(0)
    } else {
        Symbol::OneByte(0)
    };

    let mut huffman = Huffman::new(symbol);
    let compressed = huffman.compress(&file_data);

    let stats = huffman.get_compression_stats(file_data.len(), compressed.len());
    let codes = if show_codes {
        huffman.get_codes()
    } else {
        None
    };

    let output_filename = format!("{}.hsa", filename);

    let file_data_base64 = STANDARD.encode(&compressed);

    Ok(HttpResponse::Ok().json(EncodeResponse {
        success: true,
        message: "File encoded successfully".to_string(),
        filename: output_filename,
        original_size: stats.original_size,
        compressed_size: stats.compressed_size,
        header_size: stats.header_size,
        compressed_data_size: stats.compressed_data_size,
        compression_ratio: stats.compression_ratio,
        space_saved: stats.space_saved,
        percentage_saved: stats.percentage_saved,
        codes,
        file_data: file_data_base64,
    }))
}

pub async fn decode_file(
    mut payload: Multipart,
    query: web::Query<DecodingOptions>,
) -> Result<HttpResponse> {
    let mut file_data = Vec::new();
    let mut filename = String::from("unknown.hsa");
    let show_codes = query.show_codes;

    while let Some(item) = payload.next().await {
        let mut field = item?;

        let content_disposition = field.content_disposition();
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

    if file_data.is_empty() {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: "No file data received".to_string(),
        }));
    }

    if !filename.ends_with(".hsa") && !filename.ends_with(".HSA") {
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: "Invalid file extension. Only .hsa files are supported.".to_string(),
        }));
    }

    let mut huffman = Huffman::new(Symbol::OneByte(0));
    let decompressed = huffman.decompress(&file_data);

    let codes = if show_codes {
        huffman.get_codes()
    } else {
        None
    };

    let timestamp = Local::now().format("%d-%m-%Y-%H-%M").to_string();

    let base_name = filename
        .strip_suffix(".hsa")
        .or_else(|| filename.strip_suffix(".HSA"))
        .unwrap_or(&filename);

    let output_filename = if let Some(dot_pos) = base_name.rfind('.') {
        let name_part = &base_name[..dot_pos];
        let ext_part = &base_name[dot_pos..];
        format!("{}.{}{}", name_part, timestamp, ext_part)
    } else {
        format!("{}.{}", base_name, timestamp)
    };

    let file_data_base64 = STANDARD.encode(&decompressed);

    Ok(HttpResponse::Ok().json(DecodeResponse {
        success: true,
        message: "File decoded successfully".to_string(),
        filename: output_filename,
        original_size: file_data.len(),
        decompressed_size: decompressed.len(),
        codes,
        file_data: file_data_base64,
    }))
}

pub async fn index() -> Result<actix_files::NamedFile> {
    Ok(actix_files::NamedFile::open("static/index.html")?)
}
