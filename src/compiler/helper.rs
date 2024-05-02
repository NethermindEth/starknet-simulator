use crate::compiler::compile::CompilationResult;
use crate::compiler::compile_contract::ContractCompilationResult;

pub enum CompilationResultType {
    Contract(ContractCompilationResult),
    General(CompilationResult),
}

pub fn trace_error(pc: u64, compilation_result: CompilationResultType) {
    let casm_sierra_mapping = match &compilation_result {
        CompilationResultType::Contract(contract_compilation_result) => {
            &contract_compilation_result
                .casm_sierra
                .casm_sierra_mapping_instruction
                .casm_sierra_mapping
        }
        CompilationResultType::General(general_compilation_result) => {
            &general_compilation_result
                .casm_sierra
                .casm_sierra_mapping_instruction
                .casm_sierra_mapping
        }
    };

    let sierra_instruction = casm_sierra_mapping.iter().find(|(i, _)| **i == pc);

    if let Some((_, sierra_statement_indices)) = sierra_instruction {
        let sierra_cairo_info_mapping = match &compilation_result {
            CompilationResultType::Contract(contract_compilation_result) => {
                &contract_compilation_result
                    .cairo_sierra
                    .sierra_cairo_info_mapping
            }
            CompilationResultType::General(general_compilation_result) => {
                &general_compilation_result
                    .cairo_sierra
                    .sierra_cairo_info_mapping
            }
        };

        for statement_index in sierra_statement_indices {
            if let Some(cairo_info) = sierra_cairo_info_mapping.get(statement_index) {
                let cairo_locations = &cairo_info.cairo_locations;
                println!("cairo_locations: {:?}", cairo_locations);
            }
        }
    } else {
        println!("No instruction found at program counter {}", pc);
    }
}
