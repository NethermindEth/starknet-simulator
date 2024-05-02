use crate::cairo_sierra::cairo_contract::compile_contract_cairo_to_sierra;
use crate::cairo_sierra::compile::FullProgram;
use crate::casm_sierra::cairo::CasmSierraMappingInstruction;
use crate::casm_sierra::cairo_contract::conpile_contract_sierra_to_casm;
use crate::casm_sierra::cairo_contract_helper::SierraContractCompile;
use crate::compiler::helper::{trace_error, CompilationResultType};

use anyhow::{Context, Result};
use std::fs;

pub struct ContractCompilationResult {
    pub cairo_sierra: FullProgram,
    pub casm_sierra: SierraContractCompile,
}

pub fn compile_contract(
    cairo_path: String,
    sierra_path: String,
) -> Result<ContractCompilationResult> {
    // Changed return type to Result from anyhow
    let full_program = compile_contract_cairo_to_sierra(cairo_path.clone());
    let cairo_sierra = full_program.unwrap();
    let program = serde_json::to_string_pretty(&cairo_sierra.contract_class).unwrap();
    fs::write(&sierra_path, program)
        .with_context(|| format!("Failed to write output to {}", sierra_path))?; // Added format! for context

    let casm_program = conpile_contract_sierra_to_casm(sierra_path);
    let casm_program = casm_program.with_context(|| "Failed to compile CASM program")?; // Added with_context for casm_program
    Ok(ContractCompilationResult {
        cairo_sierra,
        casm_sierra: casm_program,
    })
}

pub fn main() {
    let pc_error = 67; // REPLACE_WITH_PROGRAM_COUNTER_ERROR
    let cairo_path = "Cairo_file_path.cairo".to_string();
    let sierra_path = "Sierra_file_path.json".to_string();
    let full_program = compile_contract(cairo_path, sierra_path).unwrap();
    trace_error(pc_error, CompilationResultType::Contract(full_program));
}
