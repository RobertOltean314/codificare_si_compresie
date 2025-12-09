use actix_web::{web, App, HttpServer};

use crate::api::{decode_file, encode_file, index};

mod api;
mod bit_operations;
mod huffman;
mod models;
mod tree;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting Huffman Encoding Web Server...");
    println!("Server running at: http://127.0.0.1:8080");

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/api/encode", web::post().to(encode_file))
            .route("/api/decode", web::post().to(decode_file))
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
