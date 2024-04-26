use anyhow::Context;
use cairo_lang_sierra::ProgramParser;
use cairo_lang_sierra_to_casm::compiler::{compile, SierraToCasmConfig};
use cairo_lang_sierra_to_casm::metadata::calc_metadata;
use std::collections::HashMap;
use std::fs;

pub type CasmSierraMapping = HashMap<u64, Vec<u64>>;

fn compile_sierra_to_casm(sierra_program: String) -> Result<CasmSierraMapping, anyhow::Error> {
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

    let debug_info = cairo_program.debug_info;
    let sierra_statement_info = debug_info.sierra_statement_info;

    let mut casm_sierra_mapping = HashMap::new();
    let mut sierra_statement_index = 0;
    for sierra_statement_debug_info in sierra_statement_info.iter() {
        let casm_instruction_index = sierra_statement_debug_info.instruction_idx;
        casm_sierra_mapping
            .entry(casm_instruction_index as u64)
            .or_insert_with(Vec::new)
            .push(sierra_statement_index);
        sierra_statement_index += 1;
    }
    Ok(casm_sierra_mapping)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_sierra_to_casm() {
        let sierra_program = fs::read_to_string("<REPLACE_WITH_PATH_TO_SIERRA_PROGRAM>")
            .expect("Could not read file!");

        let casm_sierra_mapping =
            compile_sierra_to_casm(sierra_program).expect("Compilation failed");
        println!("{:?}", casm_sierra_mapping);
    }
}
