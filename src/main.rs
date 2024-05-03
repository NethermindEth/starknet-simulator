pub mod cairo_sierra;
pub mod casm_sierra;
pub mod compiler;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use compiler::helper::CompilationResultType;
use serde::Deserialize;

#[derive(Deserialize)]
struct CompileInput {
    code: String,
    file_name: String,
}

fn compilation_result_to_string(result: CompilationResultType) -> String {
    match result {
        CompilationResultType::Contract(contract) => {
            return format!(
                "cairo_sierra: {:#?}, casm_sierra: {:#?}",
                contract.cairo_sierra, contract.casm_sierra
            );
        }
        CompilationResultType::General(general) => {
            return format!(
                "cairo_sierra: {:#?}, casm_sierra: {:#?}",
                general.cairo_sierra, general.casm_sierra
            );
        }
    }
}

// This function will handle POST requests to "/compile"
async fn compile_code(input: web::Json<CompileInput>) -> impl Responder {
    let result = compiler::compile::compile(&input.code, &input.file_name)
        .map(|compilation_result| {
            // Manual string conversion function
            // Need this because InstructionRepr is not serializable
            // TODO: Find a better way to do this
            serde_json::to_string(&compilation_result_to_string(
                CompilationResultType::General(compilation_result),
            ))
            .unwrap_or_else(|_| "{}".to_string())
        })
        .map_err(|e| e.to_string()); // Convert error to string which is serializable

    match result {
        Ok(json_string) => HttpResponse::Ok()
            .content_type("application/json")
            .body(json_string),
        Err(error_message) => HttpResponse::InternalServerError().json(error_message),
    }
}

// This function will handle POST requests to "/compile_contract"
async fn compile_contract_code(input: web::Json<CompileInput>) -> impl Responder {
    let result = compiler::compile_contract::compile_contract(&input.code, &input.file_name)
        .map(|compilation_result| {
            // Manual string conversion function
            // Need this because InstructionRepr is not serializable
            // TODO: Find a better way to do this
            serde_json::to_string(&compilation_result_to_string(
                CompilationResultType::Contract(compilation_result),
            ))
            .unwrap_or_else(|_| "{}".to_string())
        })
        .map_err(|e| e.to_string()); // Convert error to string which is serializable

    match result {
        Ok(json_string) => HttpResponse::Ok()
            .content_type("application/json")
            .body(json_string),
        Err(error_message) => HttpResponse::InternalServerError().json(error_message),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/compile", web::post().to(compile_code))
            .route("/compile_contract", web::post().to(compile_contract_code))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
