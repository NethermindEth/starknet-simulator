use anyhow::Context;
use cairo_lang_casm::assembler::InstructionRepr;
use cairo_lang_sierra::ProgramParser;
use cairo_lang_sierra_to_casm::compiler::{compile, CairoProgram, SierraToCasmConfig};
use cairo_lang_sierra_to_casm::metadata::calc_metadata;

use indexmap::IndexMap;
use std::fs;

pub type CasmSierraMapping = IndexMap<u64, Vec<u64>>;
#[derive(Debug, Eq, PartialEq)]
pub struct CasmInstruction {
    pub memory: String,
    pub instruction_index: usize,
    pub instruction_representation: Option<InstructionRepr>,
}
#[derive(Debug, Eq, PartialEq)]
pub struct CasmSierraMappingInstruction {
    pub casm_instructions: Vec<CasmInstruction>,
    pub casm_sierra_mapping: CasmSierraMapping,
}

pub struct SierraCompile {
    pub casm_sierra_mapping_instruction: CasmSierraMappingInstruction,
    pub casm: String,
}

pub fn compile_sierra_to_casm(path: String) -> Result<SierraCompile, anyhow::Error> {
    let sierra_program = fs::read_to_string(path).expect("Could not read file!");
    let program = ProgramParser::new()
        .parse(&sierra_program)
        .map_err(|_| anyhow::anyhow!("Failed to parse sierra program"))?;

    let cairo_program = compile(
        &program,
        &calc_metadata(&program, Default::default())
            .with_context(|| "Failed calculating Sierra variables.")?,
        SierraToCasmConfig {
            gas_usage_check: true,
            max_bytecode_size: usize::MAX,
        },
    )
    .with_context(|| "Compilation failed.")?;

    if let Ok(casm_sierra_mapping_instruction) =
        get_casm_sierra_mapping_instructions(cairo_program.clone())
    {
        Ok(SierraCompile {
            casm_sierra_mapping_instruction,
            casm: cairo_program.to_string(),
        })
    } else {
        Err(anyhow::anyhow!("Failed to compile sierra to casm"))
    }
}

pub fn get_casm_sierra_mapping_instructions(
    cairo_program: CairoProgram,
) -> Result<CasmSierraMappingInstruction, anyhow::Error> {
    let instructions = cairo_program.instructions;
    let mut casm_instructions = Vec::new();
    for (index, instruction) in instructions.iter().enumerate() {
        let instruction_representation = instruction.assemble();
        let mut first = true;
        let encoded_instructions = instruction_representation.encode();
        for encoded_instruction in encoded_instructions.iter() {
            let hex_instruction = format!("0x{:x}", encoded_instruction);
            if first {
                casm_instructions.push(CasmInstruction {
                    memory: hex_instruction,
                    instruction_representation: Some(instruction.assemble()),
                    instruction_index: index,
                });
                first = false; // Set first to false after the first iteration
            } else {
                casm_instructions.push(CasmInstruction {
                    memory: hex_instruction,
                    instruction_representation: None,
                    instruction_index: index,
                });
            }
        }
    }

    let debug_info = cairo_program.debug_info;
    let sierra_statement_info = debug_info.sierra_statement_info;

    let mut casm_sierra_mapping = IndexMap::new();
    let mut sierra_statement_index = 0;
    for sierra_statement_debug_info in sierra_statement_info.iter() {
        let casm_instruction_index = sierra_statement_debug_info.instruction_idx;
        casm_sierra_mapping
            .entry(casm_instruction_index as u64)
            .or_insert_with(Vec::new)
            .push(sierra_statement_index);
        sierra_statement_index += 1;
    }

    Ok(CasmSierraMappingInstruction {
        casm_instructions,
        casm_sierra_mapping,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_sierra_to_casm() {
        let path = "Sierra_file_path.sierra".to_string();
        let casm_sierra_mapping = compile_sierra_to_casm(path).expect("Compilation failed");
        // println!("{:?}", casm_sierra_mapping);
    }
}
