use std::path::Path;

use crate::cairo_sierra::cairo_helper::{compile_cairo_project_at_path, CompilerConfig};

pub fn compile_cairo() {
    let project_config_path = Path::new("/Users/jelilat/nethermind/hello_cairo/src/fib.cairo");
    let full_program = match compile_cairo_project_at_path(
        &project_config_path,
        CompilerConfig {
            replace_ids: true,
            ..CompilerConfig::default()
        },
    ) {
        Ok(prog) => prog,
        Err(e) => {
            eprintln!("Compilation failed: {}", e);
            return;
        }
    };
    println!(
        "sierra_cairo_info_mapping: {:?}",
        full_program.program.statements
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_compile_sierra() {
        super::compile_cairo();
    }
}