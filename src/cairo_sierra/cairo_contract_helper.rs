use std::path::{Path, PathBuf};

use anyhow::Result;
use cairo_lang_compiler::db::RootDatabase;
use cairo_lang_compiler::project::setup_project;
use cairo_lang_compiler::CompilerConfig;
use cairo_lang_defs::ids::TopLevelLanguageElementId;
use cairo_lang_filesystem::ids::CrateId;
use cairo_lang_starknet_classes::allowed_libfuncs::ListSelector;
use itertools::{Itertools};

use crate::cairo_sierra::compile::{compile_prepared_db, FullProgram};
use cairo_lang_starknet::contract::{
    find_contracts,
};
use cairo_lang_starknet::starknet_plugin_suite;

/// Compile the contract given by path.
/// Errors if there is ambiguity.
pub fn compile_path(
    path: &Path,
    contract_path: Option<&str>,
    compiler_config: CompilerConfig<'_>,
) -> Result<FullProgram> {
    let mut db = RootDatabase::builder()
        .detect_corelib()
        .with_plugin_suite(starknet_plugin_suite())
        .build()?;

    let main_crate_ids = setup_project(&mut db, Path::new(&path))?;

    compile_contract_in_prepared_db(&db, contract_path, main_crate_ids, compiler_config)
}

/// Runs StarkNet contract compiler on the specified contract.
/// If no contract was specified, verify that there is only one.
/// Otherwise, return an error.
pub fn compile_contract_in_prepared_db(
    db: &RootDatabase,
    contract_path: Option<&str>,
    main_crate_ids: Vec<CrateId>,
    mut compiler_config: CompilerConfig<'_>,
) -> Result<FullProgram> {
    let mut contracts = find_contracts(db, &main_crate_ids);

    if let Some(contract_path) = contract_path {
        contracts.retain(|contract| contract.submodule_id.full_path(db) == contract_path);
    };
    let contract = match contracts.len() {
        0 => {
            // Report diagnostics as they might reveal the reason why no contract was found.
            compiler_config.diagnostics_reporter.ensure(db)?;
            anyhow::bail!("Contract not found.");
        }
        1 => &contracts[0],
        _ => {
            let contract_names = contracts
                .iter()
                .map(|contract| contract.submodule_id.full_path(db))
                .join("\n  ");
            anyhow::bail!(
                "More than one contract found in the main crate: \n  {}\nUse --contract-path to \
                 specify which to compile.",
                contract_names
            );
        }
    };

    let contracts = vec![contract];
    let mut classes = compile_prepared_db(db, &contracts, compiler_config)?;
    assert_eq!(classes.len(), 1);
    Ok(classes.remove(0))
}

/// Compile Starknet crate (or specific contract in the crate).
pub fn starknet_compile(
    crate_path: PathBuf,
    contract_path: Option<String>,
    config: Option<CompilerConfig<'_>>,
    allowed_libfuncs_list: Option<ListSelector>,
) -> anyhow::Result<FullProgram> {
    let full_program = compile_path(
        &crate_path,
        contract_path.as_deref(),
        if let Some(config) = config {
            config
        } else {
            CompilerConfig::default()
        },
    )?;

    full_program
        .sierra_contract_class
        .validate_version_compatible(
            if let Some(allowed_libfuncs_list) = allowed_libfuncs_list {
                allowed_libfuncs_list
            } else {
                ListSelector::default()
            },
        )?;
    Ok(full_program)
}
