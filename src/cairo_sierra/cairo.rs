use std::path::Path;

use crate::cairo_sierra::cairo_helper::{compile_cairo_project_at_path, CompilerConfig};

use super::cairo_helper::FullProgram;

pub fn compile_cairo(file_path: String) -> anyhow::Result<FullProgram> {
    let project_config_path = Path::new(&file_path);
    let full_program = match compile_cairo_project_at_path(
        &project_config_path,
        CompilerConfig {
            replace_ids: true,
            ..CompilerConfig::default()
        },
    ) {
        Ok(prog) => prog,
        Err(e) => return Err(e.into()),
    };
    Ok(full_program)
}
#[cfg(test)]
mod tests {
    #[test]
    fn test_compile_sierra() {}
}
