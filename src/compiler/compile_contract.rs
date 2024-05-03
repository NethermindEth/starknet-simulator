use std::io::Write;
use tempfile::{tempdir, NamedTempFile};

use crate::cairo_sierra::cairo_contract::compile_contract_cairo_to_sierra;
use crate::cairo_sierra::compile::FullProgram;
use crate::casm_sierra::cairo_contract::conpile_contract_sierra_to_casm;
use crate::casm_sierra::cairo_contract_helper::SierraContractCompile;

use anyhow::{Context, Result};

#[derive(Debug)]
pub struct ContractCompilationResult {
    pub cairo_sierra: FullProgram,
    pub casm_sierra: SierraContractCompile,
}

pub fn compile_contract(code: &str, file_name: &str) -> Result<ContractCompilationResult> {
    // Create a temporary directory
    let dir = tempdir()?;

    let cairo_file_path = dir.path().join(format!("{}.cairo", file_name));

    // Create and write to the file
    let mut cairo_temp_file = NamedTempFile::new_in(dir.path())?;
    cairo_temp_file.write_all(code.as_bytes())?;
    cairo_temp_file.persist(&cairo_file_path)?;
    let cairo_path = cairo_file_path.to_str().unwrap().to_string();

    let full_program = compile_contract_cairo_to_sierra(cairo_path.clone());
    let cairo_sierra = full_program.unwrap();
    let program = serde_json::to_string_pretty(&cairo_sierra.contract_class).unwrap();

    let sierra_file_path = cairo_file_path.with_extension("sierra");

    let mut sierra_temp_file = NamedTempFile::new_in(dir.path())?;
    sierra_temp_file.write_all(program.as_bytes())?;
    sierra_temp_file.persist(&sierra_file_path)?;
    let sierra_path = sierra_file_path.to_str().unwrap().to_string();

    let casm_program = conpile_contract_sierra_to_casm(sierra_path);
    let casm_program = casm_program.with_context(|| "Failed to compile CASM program")?; // Added with_context for casm_program
    Ok(ContractCompilationResult {
        cairo_sierra,
        casm_sierra: casm_program,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_compile_contract() {
        let code = r#"#[starknet::interface]
        pub trait IHelloStarknet<TContractState> {
            fn increase_balance(ref self: TContractState, amount: felt252);
            fn get_balance(self: @TContractState) -> felt252;
        }
        
        #[starknet::contract]
        mod HelloStarknet {
            #[storage]
            struct Storage {
                balance: felt252, 
            }
        
            #[abi(embed_v0)]
            impl HelloStarknetImpl of super::IHelloStarknet<ContractState> {
                fn increase_balance(ref self: ContractState, amount: felt252) {
                    assert(amount != 0, 'Amount cannot be 0');
                    self.balance.write(self.balance.read() + amount);
                }
        
                fn get_balance(self: @ContractState) -> felt252 {
                    self.balance.read()
                }
            }
        }
        "#;
        let file_name = "Balance";
        super::compile_contract(code, file_name).unwrap();
    }
}
