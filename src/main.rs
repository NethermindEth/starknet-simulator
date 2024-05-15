pub mod cairo_sierra;
pub mod casm_sierra;
pub mod compiler;
pub mod trace;

use actix_cors::Cors;
use actix_web::{http, web, App, HttpResponse, HttpServer, Responder};
use cairo_lang_starknet_classes_2_point_6::casm_contract_class::CasmContractClass;
use cairo_vm::types::relocatable::MaybeRelocatable;
use starknet_types_core::felt::Felt;

use serde::Deserialize;

#[derive(Deserialize)]
struct CompileInput {
    code: String,
    file_name: String,
}

#[derive(Deserialize)]
struct TraceInput {
    args: Vec<String>,
    casm_contract_class: String,
    entrypoint_offset: usize,
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

// This function will handle POST requests to "/trace_error"
async fn trace_error(input: web::Json<TraceInput>) -> impl Responder {
    let casm_contract_class = serde_json::from_str::<CasmContractClass>(&input.casm_contract_class)
        .expect("Failed to parse casm_contract_class");

    let args: Vec<Felt> = input
        .args
        .iter()
        .map(|arg| Felt::from_hex_unchecked(arg))
        .collect();

    let relocated_args: Vec<MaybeRelocatable> = args.iter().map(MaybeRelocatable::from).collect();

    let result = trace::cairo_runner::trace_error(
        casm_contract_class,
        input.entrypoint_offset,
        &relocated_args,
    );

    match result {
        Ok(trace_result) => HttpResponse::Ok().json(trace_result),
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
            .route("/trace_error", web::post().to(trace_error))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
