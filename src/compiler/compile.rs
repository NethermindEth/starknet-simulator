use crate::cairo_sierra::cairo::compile_cairo;
use crate::cairo_sierra::cairo_helper::FullProgram;
use crate::casm_sierra::cairo::{compile_sierra_to_casm, SierraCompile};
use crate::compiler::helper::{trace_error, CompilationResultType};

use anyhow::{Context, Result};
use std::fs;

pub struct CompilationResult {
    pub cairo_sierra: FullProgram,
    pub casm_sierra: SierraCompile,
}

pub fn compile(cairo_path: String, sierra_path: String) -> Result<CompilationResult> {
    let full_program = compile_cairo(cairo_path.clone());
    let cairo_sierra = full_program.unwrap();
    fs::write(&sierra_path, format!("{}", cairo_sierra.program))
        .context("Failed to write output to {}")?;

    let casm_program = compile_sierra_to_casm(sierra_path);
    let casm_program = casm_program.with_context(|| "Failed to compile CASM program")?; // Added with_context for casm_program
    Ok(CompilationResult {
        cairo_sierra,
        casm_sierra: casm_program,
    })
}

pub fn main() {
    let pc_error = 8; // REPLACE_WITH_PROGRAM_COUNTER_ERROR
    let cairo_path = "Cairo_file_path.cairo".to_string();
    let sierra_path = "Sierra_file_path.sierra".to_string();
    let full_program = compile(cairo_path, sierra_path).unwrap();
    trace_error(pc_error, CompilationResultType::General(full_program));
}
