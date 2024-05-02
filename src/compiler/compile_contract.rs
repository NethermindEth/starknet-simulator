use crate::cairo_sierra::cairo_contract::compile_contract_cairo_to_sierra;
use crate::cairo_sierra::compile::FullProgram;
use crate::casm_sierra::cairo::CasmSierraMappingInstruction;
use crate::casm_sierra::cairo_contract::conpile_contract_sierra_to_casm;
use crate::casm_sierra::cairo_contract_helper::SierraContractCompile;

use anyhow::{Context, Result};
use std::fs; // Added use statement for anyhow

pub struct CompilationResult {
    pub cairo_sierra: FullProgram,
    pub casm_sierra: SierraContractCompile,
}

pub fn compile_contract(cairo_path: String, sierra_path: String) -> Result<CompilationResult> {
    // Changed return type to Result from anyhow
    let full_program = compile_contract_cairo_to_sierra(cairo_path.clone());
    let cairo_sierra = full_program.unwrap();
    let program = serde_json::to_string_pretty(&cairo_sierra.contract_class).unwrap();
    fs::write(&sierra_path, program)
        .with_context(|| format!("Failed to write output to {}", sierra_path))?; // Added format! for context

    let casm_program = conpile_contract_sierra_to_casm(sierra_path);
    let casm_program = casm_program.with_context(|| "Failed to compile CASM program")?; // Added with_context for casm_program
    Ok(CompilationResult {
        cairo_sierra,
        casm_sierra: casm_program,
    })
}

pub fn trace_error(pc: u64, compilation_result: CompilationResult) {
    let casm_sierra_mapping = compilation_result
        .casm_sierra
        .casm_sierra_mapping_instruction
        .casm_sierra_mapping;
    let sierra_instruction = casm_sierra_mapping.iter().find(|(i, _)| **i == pc);
    // println!("sierra_instruction: {:?}", sierra_instruction);
    if let Some((_, sierra_statement_indices)) = sierra_instruction {
        let sierra_cairo_info_mapping = compilation_result.cairo_sierra.sierra_cairo_info_mapping;
        for statement_index in sierra_statement_indices {
            for (index, cairo_info) in &sierra_cairo_info_mapping {
                if index == statement_index {
                    let cairo_locations = &cairo_info.cairo_locations;
                    println!("cairo_locations: {:?}", cairo_locations);
                    // for location in cairo_locations {
                    //     if location.file_name == "Balance.cairo" {
                    //         println!("Location: {:?}", location);
                    //     }
                    // }
                }
            }
        }
    } else {
        println!("No instruction found at program counter {}", pc);
    }
}
pub fn main() {
    let pc_error = 67; // REPLACE_WITH_PROGRAM_COUNTER_ERROR
    let cairo_path = "Cairo_file_path.cairo".to_string();
    let sierra_path = "Sierra_file_path.json".to_string();
    let full_program = compile_contract(cairo_path, sierra_path).unwrap();
    trace_error(pc_error, full_program);
}
