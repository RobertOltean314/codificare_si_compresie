use actix_cors::Cors;
use actix_web::{App, HttpServer, web};

use crate::api::{decode_file, encode_file, index};

mod api;
mod bit_operations;
mod lzw;
mod models;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("LZW Compression Server starting...");
    println!("Server running at http://127.0.0.1:8080");

    HttpServer::new(|| {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .route("/", web::get().to(index))
            .route("/api/encode", web::post().to(encode_file))
            .route("/api/decode", web::post().to(decode_file))
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
