//! Cairo compiler.
//!
//! This crate is responsible for compiling a Cairo project into a Sierra program.
//! It is the main entry point for the compiler.
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Write, path::Path, sync::Arc};

use ::cairo_lang_diagnostics::ToOption;
use anyhow::{Context, Result};
use cairo_lang_defs::db::DefsGroup;
use cairo_lang_diagnostics::DiagnosticLocation;
use cairo_lang_filesystem::{
    db::FilesGroup,
    ids::{CrateId, FileLongId},
};
use cairo_lang_sierra::program::{Program, StatementIdx};
use cairo_lang_sierra_generator::statements_locations::StatementsLocations;
use cairo_lang_sierra_generator::{db::SierraGenGroup, replace_ids::replace_sierra_ids_in_program};
use cairo_lang_utils::unordered_hash_map::UnorderedHashMap;

use cairo_lang_compiler::{
    db::RootDatabase,
    diagnostics::DiagnosticsReporter,
    project::{get_main_crate_ids_from_project, setup_project, ProjectConfig},
};

/// Configuration for the compiler.
#[derive(Default)]
pub struct CompilerConfig<'c> {
    pub diagnostics_reporter: DiagnosticsReporter<'c>,

    /// Replaces sierra ids with human-readable ones.
    pub replace_ids: bool,

    /// The name of the allowed libfuncs list to use in compilation.
    /// If None the default list of audited libfuncs will be used.
    pub allowed_libfuncs_list_name: Option<String>,

    /// Adds mapping used by [cairo-profiler](https://github.com/software-mansion/cairo-profiler) to
    /// [cairo_lang_sierra::debug_info::Annotations] in [cairo_lang_sierra::debug_info::DebugInfo].
    pub add_statements_functions: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextPosition {
    /// Line index, 0 based.
    pub line: usize,
    /// Character index inside the line, 0 based.
    pub col: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CairoLocation {
    pub file_name: String,
    pub start: TextPosition,
    pub end: TextPosition,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CairoInfo {
    pub fn_name: String,
    pub cairo_locations: Option<Vec<CairoLocation>>,
}
pub type SierraCairoInfoMapping = IndexMap<u64, CairoInfo>;

#[derive(Debug, Serialize, Deserialize)]
pub struct FullProgram {
    pub contract: String,
    pub program: Program,
    pub sierra_cairo_info_mapping: SierraCairoInfoMapping,
}

/// Compiles a Cairo project at the given path.
/// The project must be a valid Cairo project:
/// Either a standalone `.cairo` file (a single crate), or a directory with a `cairo_project.toml`
/// file.
/// # Arguments
/// * `path` - The path to the project.
/// * `compiler_config` - The compiler configuration.
/// # Returns
/// * `Ok(Program)` - The compiled program.
/// * `Err(anyhow::Error)` - Compilation failed.
pub fn compile_cairo_project_at_path(
    path: &Path,
    compiler_config: CompilerConfig<'_>,
) -> Result<FullProgram> {
    let mut db = RootDatabase::builder().detect_corelib().build()?;
    let main_crate_ids = setup_project(&mut db, path)?;
    compile_prepared_db_program(&mut db, main_crate_ids, compiler_config)
}

/// Compiles a Cairo project.
/// The project must be a valid Cairo project.
/// This function is a wrapper over [`RootDatabase::builder()`] and [`compile_prepared_db_program`].
/// # Arguments
/// * `project_config` - The project configuration.
/// * `compiler_config` - The compiler configuration.
/// # Returns
/// * `Ok(Program)` - The compiled program.
/// * `Err(anyhow::Error)` - Compilation failed.
pub fn compile(
    project_config: ProjectConfig,
    compiler_config: CompilerConfig<'_>,
) -> Result<FullProgram> {
    let mut db = RootDatabase::builder()
        .with_project_config(project_config.clone())
        .build()?;
    let main_crate_ids = get_main_crate_ids_from_project(&mut db, &project_config);

    compile_prepared_db_program(&mut db, main_crate_ids, compiler_config)
}

/// Runs Cairo compiler.
///
/// # Arguments
/// * `db` - Preloaded compilation database.
/// * `main_crate_ids` - [`CrateId`]s to compile. Do not include dependencies here, only pass
///   top-level crates in order to eliminate unused code. Use
///   `db.intern_crate(CrateLongId::Real(name))` in order to obtain [`CrateId`] from its name.
/// * `compiler_config` - The compiler configuration.
/// # Returns
/// * `Ok(FullProgram)` - The compiled program and additional info.
/// * `Err(anyhow::Error)` - Compilation failed.
pub fn compile_prepared_db_program(
    db: &mut RootDatabase,
    main_crate_ids: Vec<CrateId>,
    compiler_config: CompilerConfig<'_>,
) -> Result<FullProgram> {
    match compile_prepared_db(db, main_crate_ids, compiler_config) {
        Ok(sierra_program_with_debug) => {
            let statement_locations = sierra_program_with_debug.statement_locations;
            let statements_functions_map = statement_locations.get_statements_functions_map(db);

            let diagnostic_locations = get_diagnostic_locations(db, statement_locations);

            let sierra_cairo_info_mapping = generate_sierra_to_cairo_statement_info(
                db,
                sierra_program_with_debug.program.statements.len() as usize,
                statements_functions_map,
                diagnostic_locations,
            );

            match sierra_cairo_info_mapping {
                Ok(mapping) => Ok(FullProgram {
                    contract: mapping.contract_code,
                    program: sierra_program_with_debug.program,
                    sierra_cairo_info_mapping: mapping.sierra_cairo_statement_info,
                }),
                Err(e) => Err(e.into()),
            }
        }
        Err(e) => Err(e),
    }
}

pub fn get_diagnostic_locations(
    db: &dyn DefsGroup,
    statement_locations: StatementsLocations,
) -> IndexMap<StatementIdx, Vec<DiagnosticLocation>> {
    statement_locations
        .locations
        .iter_sorted()
        .map(|(statement_idx, location)| (*statement_idx, location.diagnostic_location(db)))
        .collect::<Vec<_>>()
        .into_iter()
        .fold(
            IndexMap::new(),
            |mut acc, (statement_idx, diagnostic_location)| {
                acc.entry(statement_idx)
                    .or_insert_with(Vec::new)
                    .push(diagnostic_location);
                acc
            },
        )
}

pub struct SierraCairoStatement {
    pub contract_code: String,
    pub sierra_cairo_statement_info: SierraCairoInfoMapping,
}

pub struct SierraProgramWithDebug {
    pub program: Program,
    pub statement_locations: StatementsLocations,
}

// Generates mapping information between Sierra and Cairo statements
//
// # Arguments
// * `db` - Preloaded compilation database.
// * `no_of_statements` - The number of statements in the program.
// * `statements_functions_map` - The map of statement to function name.
// * `diagnostic_locations` - The map of statement to diagnostic location.
// # Returns
// * `SierraCairoInfoMapping` - The map of statement to Cairo info.
pub fn generate_sierra_to_cairo_statement_info(
    db: &dyn FilesGroup,
    no_of_statements: usize,
    statements_functions_map: UnorderedHashMap<StatementIdx, String>,
    diagnostic_locations: IndexMap<StatementIdx, Vec<DiagnosticLocation>>,
) -> Result<SierraCairoStatement, std::io::Error> {
    let mut sierra_cairo_info_mapping: SierraCairoInfoMapping = IndexMap::new();
    let mut contract_content: String = String::new();

    for idx in 0..no_of_statements {
        let statement_idx = StatementIdx(idx);
        let idx_u64 = idx as u64; // Convert idx to u64
        if let Some(function_name) = statements_functions_map.get(&statement_idx) {
            if let Some(info) = sierra_cairo_info_mapping.get_mut(&idx_u64) {
                info.fn_name = function_name.clone();
            } else {
                sierra_cairo_info_mapping.insert(
                    idx_u64,
                    CairoInfo {
                        fn_name: function_name.clone(),
                        cairo_locations: None,
                    },
                );
            }
        }
        if let Some(locations) = diagnostic_locations.get(&statement_idx) {
            if let Some(info) = sierra_cairo_info_mapping.get_mut(&idx_u64) {
                let cairo_locations = &mut info.cairo_locations;
                for location in locations {
                    let file_id = location.file_id;
                    let file_name = file_id.file_name(db);
                    if contract_content.is_empty() && file_name == "contract" {
                        match db.lookup_intern_file(file_id) {
                            FileLongId::Virtual(vf) => {
                                contract_content = vf.content.to_string();
                            }
                            _ => {}
                        }
                    }

                    let start_offset = location.span.start;
                    let start_position_in_file = start_offset.position_in_file(db, file_id);
                    let start_position = match start_position_in_file {
                        Some(position) => TextPosition {
                            line: position.line,
                            col: position.col,
                        },
                        None => TextPosition { line: 0, col: 0 },
                    };

                    let end_offset = location.span.end;
                    let end_position_in_file = end_offset.position_in_file(db, file_id);
                    let end_position = match end_position_in_file {
                        Some(position) => TextPosition {
                            line: position.line,
                            col: position.col,
                        },
                        None => TextPosition { line: 0, col: 0 },
                    };
                    if let Some(locations) = cairo_locations {
                        locations.push(CairoLocation {
                            file_name,
                            start: start_position,
                            end: end_position,
                        });
                    } else {
                        *cairo_locations = Some(vec![CairoLocation {
                            file_name,
                            start: start_position,
                            end: end_position,
                        }]);
                    }
                }
            }
        }
    }

    Ok(SierraCairoStatement {
        contract_code: contract_content,
        sierra_cairo_statement_info: sierra_cairo_info_mapping,
    })
}

/// Runs Cairo compiler.
///
/// Similar to `compile_prepared_db_program`, but this function returns all the raw debug
/// information.
///
/// # Arguments
/// * `db` - Preloaded compilation database.
/// * `main_crate_ids` - [`CrateId`]s to compile. Do not include dependencies here, only pass
///   top-level crates in order to eliminate unused code. Use
///   `db.intern_crate(CrateLongId::Real(name))` in order to obtain [`CrateId`] from its name.
/// * `compiler_config` - The compiler configuration.
/// # Returns
/// * `Ok(SierraProgramWithDebug)` - The compiled program with debug info.
/// * `Err(anyhow::Error)` - Compilation failed.
pub fn compile_prepared_db(
    db: &mut RootDatabase,
    main_crate_ids: Vec<CrateId>,
    mut compiler_config: CompilerConfig<'_>,
) -> Result<SierraProgramWithDebug> {
    compiler_config.diagnostics_reporter.ensure(db)?;

    let arc_result = db
        .get_sierra_program(main_crate_ids)
        .to_option()
        .context("Compilation failed without any diagnostics")?;

    let (program_arc, statements_locations_arc) = arc_result;

    let mut program = Arc::try_unwrap(program_arc).unwrap_or_else(|arc| (*arc).clone());
    let statements_locations =
        Arc::try_unwrap(statements_locations_arc).unwrap_or_else(|arc| (*arc).clone());

    if compiler_config.replace_ids {
        program = replace_sierra_ids_in_program(db, &program).into();
    }
    Ok(SierraProgramWithDebug {
        program,
        statement_locations: statements_locations,
    })
}
