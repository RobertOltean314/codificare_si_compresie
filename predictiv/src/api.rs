use std::fmt::write;

use crate::{bit_operations::BitWriter, predictiv::Predictiv};
use actix_files::NamedFile;
use actix_web::{HttpResponse, Result, web};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct EncodeResponse {
    pub encoded_filename: String,
    pub original_image: Vec<Vec<i32>>,
    pub error_matrix: Vec<Vec<i32>>,
    pub prediction_type: u8,
    pub encoded_data: Vec<u8>,
}

#[derive(Debug, Deserialize)]
pub struct EncodeRequest {
    pub file_name: String,
    pub file_data: Vec<u8>,
    pub prediction_number: u8,
}

pub async fn encode_file(req: web::Json<EncodeRequest>) -> Result<HttpResponse, actix_web::Error> {
    let mut writer = BitWriter::new();

    writer.data.extend_from_slice(&req.file_data[0..1078]);
    writer.write_n_bits(4, req.prediction_number as u32);

    let original =
        Predictiv::read_image(&req.file_data).map_err(actix_web::error::ErrorBadRequest)?;
    let error_predictiv = original.predict(req.prediction_number as usize);
    let encoded_filename = format!("{}[{}].pre", req.file_name, req.prediction_number);

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
    };

    Ok(HttpResponse::Ok().json(response))
}

pub async fn decode_file() -> Result<HttpResponse, actix_web::Error> {
    // TODO: implement later
    Ok(HttpResponse::NotImplemented().finish())
}

pub async fn index() -> Result<NamedFile> {
    Ok(NamedFile::open("static/index.html")?)
}
