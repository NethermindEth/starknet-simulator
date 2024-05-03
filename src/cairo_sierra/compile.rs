use std::sync::Arc;

use anyhow::{Context, Result};
use cairo_felt::Felt252;
use cairo_lang_compiler::db::RootDatabase;
use cairo_lang_compiler::CompilerConfig;
use cairo_lang_diagnostics::ToOption;
use cairo_lang_lowering::db::LoweringGroup;
use cairo_lang_lowering::ids::ConcreteFunctionWithBodyId;
use cairo_lang_sierra::debug_info::Annotations;
use cairo_lang_sierra::ids::FunctionId;
use cairo_lang_sierra_generator::canonical_id_replacer::CanonicalReplacer;
use cairo_lang_sierra_generator::db::SierraGenGroup;
use cairo_lang_sierra_generator::program_generator::SierraProgramWithDebug;
use cairo_lang_sierra_generator::replace_ids::{replace_sierra_ids_in_program, SierraIdReplacer};
use cairo_lang_starknet_classes::contract_class::{
    ContractClass, ContractEntryPoint, ContractEntryPoints,
};
use cairo_lang_starknet_classes::keccak::starknet_keccak;
use cairo_lang_utils::Intern;
use itertools::{chain, Itertools};

use crate::cairo_sierra::aliased::Aliased;
use crate::cairo_sierra::cairo_helper::{
    generate_sierra_to_cairo_statement_info, get_diagnostic_locations,
};
use cairo_lang_starknet::abi::AbiBuilder;
use cairo_lang_starknet::contract::{
    find_contracts, get_contract_abi_functions, ContractDeclaration,
};
use cairo_lang_starknet::plugin::consts::{CONSTRUCTOR_MODULE, EXTERNAL_MODULE, L1_HANDLER_MODULE};

use super::cairo_helper::SierraCairoInfoMapping;

#[derive(Debug)]
pub struct FullProgram {
    pub contract_class: ContractClass,
    pub sierra_cairo_info_mapping: SierraCairoInfoMapping,
}

/// Runs Starknet contracts compiler.
///
/// # Arguments
/// * `db` - Preloaded compilation database.
/// * `contracts` - [`ContractDeclaration`]s to compile. Use [`find_contracts`] to find contracts in
///   `db`.
/// * `compiler_config` - The compiler configuration.
/// # Returns
/// * `Ok(Vec<ContractClass>)` - List of all compiled contract classes found in main cairo_lang_starknets.
/// * `Err(anyhow::Error)` - Compilation failed.
pub fn compile_prepared_db(
    db: &RootDatabase,
    contracts: &[&ContractDeclaration],
    mut compiler_config: CompilerConfig<'_>,
) -> Result<Vec<FullProgram>> {
    compiler_config.diagnostics_reporter.ensure(db)?;

    contracts
        .iter()
        .map(|contract| {
            compile_contract_with_prepared_and_checked_db(db, contract, &compiler_config)
        })
        .try_collect()
}

/// Compile declared Starknet contract.
///
/// The `contract` value **must** come from `db`, for example as a result of calling
/// [`find_contracts`]. Does not check diagnostics, it is expected that they are checked by caller
/// of this function.
fn compile_contract_with_prepared_and_checked_db(
    db: &RootDatabase,
    contract: &ContractDeclaration,
    compiler_config: &CompilerConfig<'_>,
) -> Result<FullProgram> {
    let SemanticEntryPoints {
        external,
        l1_handler,
        constructor,
    } = extract_semantic_entrypoints(db, contract)?;
    let SierraProgramWithDebug {
        program: mut sierra_program,
        debug_info,
    } = Arc::unwrap_or_clone(
        db.get_sierra_program_for_functions(
            chain!(&external, &l1_handler, &constructor)
                .map(|f| f.value)
                .collect(),
        )
        .to_option()
        .with_context(|| "Compilation failed without any diagnostics.")?,
    );

    let statement_locations = debug_info.clone().statements_locations;
    let statements_functions_map = statement_locations.get_statements_functions_map_for_tests(db);

    let diagnostic_locations = get_diagnostic_locations(db, statement_locations);

    let sierra_cairo_info_mapping = generate_sierra_to_cairo_statement_info(
        db,
        sierra_program.statements.len() as usize,
        statements_functions_map,
        diagnostic_locations,
    );

    if compiler_config.replace_ids {
        sierra_program = replace_sierra_ids_in_program(db, &sierra_program);
    }
    let replacer = CanonicalReplacer::from_program(&sierra_program);
    let sierra_program = replacer.apply(&sierra_program);

    let entry_points_by_type = ContractEntryPoints {
        external: get_entry_points(db, &external, &replacer)?,
        l1_handler: get_entry_points(db, &l1_handler, &replacer)?,
        // Later generation of ABI verifies that there is up to one constructor.
        constructor: get_entry_points(db, &constructor, &replacer)?,
    };

    let annotations = if compiler_config.add_statements_functions {
        let statements_functions = debug_info
            .statements_locations
            .extract_statements_functions(db);
        Annotations::from(statements_functions)
    } else {
        Default::default()
    };

    let contract_class = ContractClass::new(
        &sierra_program,
        entry_points_by_type,
        Some(
            AbiBuilder::from_submodule(db, contract.submodule_id, Default::default())
                .ok()
                .with_context(|| "Unexpected error while generating ABI.")?
                .finalize()
                .with_context(|| "Could not create ABI from contract submodule")?,
        ),
        annotations,
    )?;
    contract_class.sanity_check();
    Ok(FullProgram {
        contract_class,
        sierra_cairo_info_mapping,
    })
}

pub struct SemanticEntryPoints {
    pub external: Vec<Aliased<ConcreteFunctionWithBodyId>>,
    pub l1_handler: Vec<Aliased<ConcreteFunctionWithBodyId>>,
    pub constructor: Vec<Aliased<ConcreteFunctionWithBodyId>>,
}

/// Extracts functions from the contract.
pub fn extract_semantic_entrypoints(
    db: &dyn LoweringGroup,
    contract: &ContractDeclaration,
) -> core::result::Result<SemanticEntryPoints, anyhow::Error> {
    let external: Vec<_> = get_contract_abi_functions(db.upcast(), contract, EXTERNAL_MODULE)?
        .into_iter()
        .map(|f| f.map(|f| ConcreteFunctionWithBodyId::from_semantic(db, f)))
        .collect();
    let l1_handler: Vec<_> = get_contract_abi_functions(db.upcast(), contract, L1_HANDLER_MODULE)?
        .into_iter()
        .map(|f| f.map(|f| ConcreteFunctionWithBodyId::from_semantic(db, f)))
        .collect();
    let constructor: Vec<_> =
        get_contract_abi_functions(db.upcast(), contract, CONSTRUCTOR_MODULE)?
            .into_iter()
            .map(|f| f.map(|f| ConcreteFunctionWithBodyId::from_semantic(db, f)))
            .collect();
    if constructor.len() > 1 {
        anyhow::bail!("Expected at most one constructor.");
    }
    let external = external
        .into_iter()
        .map(|f| Aliased::new(f.value, f.alias))
        .collect();
    let l1_handler = l1_handler
        .into_iter()
        .map(|f| Aliased::new(f.value, f.alias))
        .collect();
    let constructor = constructor
        .into_iter()
        .map(|f| Aliased::new(f.value, f.alias))
        .collect();
    Ok(SemanticEntryPoints {
        external,
        l1_handler,
        constructor,
    })
}

/// Returns the entry points given their IDs sorted by selectors.
fn get_entry_points(
    db: &RootDatabase,
    entry_point_functions: &[Aliased<ConcreteFunctionWithBodyId>],
    replacer: &CanonicalReplacer,
) -> Result<Vec<ContractEntryPoint>> {
    let mut entry_points = vec![];
    for function_with_body_id in entry_point_functions {
        let (selector, sierra_id) =
            get_selector_and_sierra_function(db, function_with_body_id, replacer);

        entry_points.push(ContractEntryPoint {
            selector: selector.to_biguint(),
            function_idx: sierra_id.id as usize,
        });
    }
    entry_points.sort_by(|a, b| a.selector.cmp(&b.selector));
    Ok(entry_points)
}

/// Converts a function to a Sierra function.
/// Returns the selector and the sierra function id.
pub fn get_selector_and_sierra_function<T: SierraIdReplacer>(
    db: &dyn SierraGenGroup,
    function_with_body: &Aliased<ConcreteFunctionWithBodyId>,
    replacer: &T,
) -> (Felt252, FunctionId) {
    let function_id = function_with_body
        .value
        .function_id(db.upcast())
        .expect("Function error.");
    let sierra_id = replacer.replace_function_id(&function_id.intern(db));
    let selector: Felt252 = starknet_keccak(function_with_body.alias.as_bytes()).into();
    (selector, sierra_id)
}
