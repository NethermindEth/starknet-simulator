use std::path::PathBuf;

// use crate::cairo_integration::cairo_contract_helper::starknet_compile;
use crate::cairo_sierra::cairo_contract_helper::starknet_compile;
use cairo_lang_compiler::diagnostics::DiagnosticsReporter;
use cairo_lang_compiler::CompilerConfig;
use cairo_lang_starknet_classes::allowed_libfuncs::ListSelector;

pub fn compile_sierra() {
    let crate_path = PathBuf::from("/Users/jelilat/nethermind/hello_cairo/src/Storage.cairo");
    // let project_config_path = Path::new();
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
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create("output.txt").expect("Failed to create file");
        writeln!(file, "{:?}", full_program.sierra_cairo_info_mapping)
            .expect("Failed to write to file");
    } else {
        // Handle the error case
        eprintln!("Compilation failed");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_compile_sierra() {
        super::compile_sierra();
    }
}
