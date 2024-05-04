pub mod cairo_sierra;
pub mod casm_sierra;
pub mod compiler;

use actix_cors::Cors;
use actix_web::{http, web, App, HttpResponse, HttpServer, Responder};

use compiler::helper::CompilationResultType;
use serde::Deserialize;

#[derive(Deserialize)]
struct CompileInput {
    code: String,
    file_name: String,
}

// This function will handle POST requests to "/compile"
async fn compile_code(input: web::Json<CompileInput>) -> impl Responder {
    let result = compiler::compile::compile(&input.code, &input.file_name);
    match result {
        Ok(compilation_result) => HttpResponse::Ok().json(compilation_result),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

// This function will handle POST requests to "/compile_contract"
async fn compile_contract_code(input: web::Json<CompileInput>) -> impl Responder {
    let result = compiler::compile_contract::compile_contract(&input.code, &input.file_name);
    match result {
        Ok(compilation_result) => HttpResponse::Ok().json(compilation_result),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000") // Adjust this to match your frontend's URL
            .allowed_methods(vec!["GET", "POST"]) // Methods you want to allow
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .wrap(cors)
            .route("/compile", web::post().to(compile_code))
            .route("/compile_contract", web::post().to(compile_contract_code))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
