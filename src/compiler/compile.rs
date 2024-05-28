use serde::{Deserialize, Serialize};
use std::io::Write;
use tempfile::{tempdir, NamedTempFile};

use crate::cairo_sierra::cairo::compile_cairo;
use crate::cairo_sierra::cairo_helper::FullProgram;
use crate::casm_sierra::cairo::{compile_sierra_to_casm, SierraCompile};

use anyhow::{Context, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct CompilationResult {
    pub cairo_sierra: FullProgram,
    pub casm_sierra: SierraCompile,
}

pub fn compile(code: &str, file_name: &str) -> Result<CompilationResult> {
    // Create a temporary directory
    let dir = tempdir()?;

    let file_name = if file_name.ends_with(".cairo") {
        file_name.replace(".cairo", "")
    } else {
        file_name.to_string()
    };

    let cairo_file_path = dir.path().join(format!("{}.cairo", file_name));

    // Create and write to the file
    let mut cairo_temp_file = NamedTempFile::new_in(dir.path())?;
    cairo_temp_file.write_all(code.as_bytes())?;
    cairo_temp_file.persist(&cairo_file_path)?;
    let cairo_path = cairo_file_path.to_str().unwrap().to_string();

    let full_program = compile_cairo(cairo_path);
    let cairo_sierra = full_program?;

    let sierra_file_path = cairo_file_path.with_extension("sierra");

    let mut sierra_temp_file = NamedTempFile::new_in(dir.path())?;
    sierra_temp_file.write_all(format!("{}", cairo_sierra.program).as_bytes())?;
    sierra_temp_file.persist(&sierra_file_path)?;
    let sierra_path = sierra_file_path.to_str().unwrap().to_string();

    let casm_program = compile_sierra_to_casm(sierra_path);
    let casm_program = casm_program.with_context(|| "Failed to compile CASM program")?; // Added with_context for casm_program
    Ok(CompilationResult {
        cairo_sierra,
        casm_sierra: casm_program,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_compile() {
        let code = r#"use core::felt252;

        fn main() -> felt252 {
            let n = 10;
            let result = fib(1, 1, n);
            result
        }
        
        fn fib(a: felt252, b: felt252, n: felt252) -> felt252 {
            match n {
                0 => a,
                _ => fib(b, a + b, n - 1),
            }
        }
        "#;
        let file_name = "fib";
        super::compile(code, file_name).unwrap();
    }
}
