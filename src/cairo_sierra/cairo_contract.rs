use std::path::PathBuf;

use crate::cairo_sierra::cairo_contract_helper::starknet_compile;
use cairo_lang_compiler::diagnostics::DiagnosticsReporter;
use cairo_lang_compiler::CompilerConfig;
use cairo_lang_starknet_classes::allowed_libfuncs::ListSelector;

use super::compile::FullProgram;

pub fn compile_contract_cairo_to_sierra(file_path: String) -> anyhow::Result<FullProgram> {
    let crate_path = PathBuf::from(&file_path);
    let list_selector = ListSelector::new(None, None)
        .expect("Both allowed libfunc list name and file were supplied.");
    let mut diagnostics_reporter = DiagnosticsReporter::stderr();
    diagnostics_reporter = diagnostics_reporter.allow_warnings();
    if let Ok(full_program) = starknet_compile(
        crate_path,
        None,
        Some(CompilerConfig {
            replace_ids: true,
            diagnostics_reporter,
            ..CompilerConfig::default()
        }),
        Some(list_selector),
    ) {
        Ok(full_program)
    } else {
        return Err(anyhow::Error::msg("Failed to compile the Cairo contract."));
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_compile_sierra() {}
}
