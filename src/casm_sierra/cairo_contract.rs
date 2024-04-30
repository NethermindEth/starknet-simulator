use std::fs;

use crate::casm_sierra::cairo_contract_helper::{CasmContractClass, SierraContractCompile};
use anyhow::Context;
use cairo_lang_starknet_classes::allowed_libfuncs::ListSelector;
use cairo_lang_starknet_classes::contract_class::{ContractClass, ContractEntryPoints};
use cairo_lang_utils::bigint::BigUintAsHex;
use clap::Parser;
use serde::Deserialize;

/// Same as `ContractClass` - but ignores `abi` in deserialization.
/// Enables loading old contract classes.
#[derive(Deserialize)]
pub struct ContractClassIgnoreAbi {
    pub sierra_program: Vec<BigUintAsHex>,
    pub sierra_program_debug_info: Option<cairo_lang_sierra::debug_info::DebugInfo>,
    pub contract_class_version: String,
    pub entry_points_by_type: ContractEntryPoints,
    pub _abi: Option<serde_json::Value>,
}

pub fn conpile_contract_sierra_to_casm(
    file_path: String,
) -> anyhow::Result<(SierraContractCompile)> {
    let list_selector = ListSelector::DefaultList;
    let ContractClassIgnoreAbi {
        sierra_program,
        sierra_program_debug_info,
        contract_class_version,
        entry_points_by_type,
        _abi,
    } = serde_json::from_str(
        &fs::read_to_string(file_path).with_context(|| "Failed to read file.")?,
    )
    .with_context(|| "Deserialization failed.")?;
    let contract_class = ContractClass {
        sierra_program,
        sierra_program_debug_info,
        contract_class_version,
        entry_points_by_type,
        abi: None,
    };
    contract_class.validate_version_compatible(list_selector)?;
    let casm_contract = CasmContractClass::from_contract_class(contract_class, false, 180000)
        .with_context(|| "Compilation failed.")?;

    Ok(casm_contract)
}

#[cfg(test)]
mod tests {
    use super::*;
}
