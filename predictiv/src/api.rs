use crate::{
    bit_operations::{BitReader, BitWriter},
    helpers::*,
    models::*,
};

use actix_files::NamedFile;
use actix_web::{HttpResponse, Result, web};

pub async fn encode_file(req: web::Json<EncodeRequest>) -> Result<HttpResponse, actix_web::Error> {
    let mut writer = BitWriter::new();

    writer.data.extend_from_slice(&req.file_data[0..1078]);
    writer.write_n_bits(4, req.prediction_number as u32);

    let original = read_image_data(&req.file_data).map_err(actix_web::error::ErrorBadRequest)?;
    let error_predictiv = predict(&original, req.prediction_number as usize);
    let encoded_filename = format!("{}[{}].pre", req.file_name, req.prediction_number);

    let original_histogram = Histogram(compute_histogram(&original.data, false));
    let error_histogram = Histogram(compute_histogram(&error_predictiv.data, true));

    for y in 0..256 {
        for x in 0..256 {
            let err = error_predictiv.data[y][x];
            let err_i8 = err.clamp(-128, 127) as i8;
            writer.write_n_bits(8, err_i8 as u32);
        }
    }

    let encoded_data = writer.finish();

    let response = EncodeResponse {
        encoded_filename,
        original_image: original.data,
        error_matrix: error_predictiv.data,
        prediction_type: req.prediction_number,
        encoded_data,
        original_histogram,
        error_histogram,
    };

    Ok(HttpResponse::Ok().json(response))
}

pub async fn decode_file(req: web::Json<DecodeRequest>) -> Result<HttpResponse, actix_web::Error> {
    let bytes = &req.file_data;

    if bytes.len() < 1079 + 65536 {
        return Err(actix_web::error::ErrorBadRequest("Invalid .pre file"));
    }

    let mut reader = BitReader::new(bytes);

    let header = get_header_data(&mut reader)?;
    let prediction_type = get_prediction_type(&mut reader)?;
    let error_data = get_error_data(&mut reader)?;
    let decoded = reconstruct_image(&error_data, prediction_type)?;

    let decoded_bmp = build_bmp(&header, &decoded);

    let response = DecodeResponse {
        decoded_filename: format!("{}.decoded.bmp", req.file_name),
        decoded_image: decoded,
        predicted_type: prediction_type as u8,
        decoded_bmp_data: decoded_bmp,
    };

    Ok(HttpResponse::Ok().json(response))
}

pub async fn index() -> Result<NamedFile> {
    Ok(NamedFile::open("static/index.html")?)
}
