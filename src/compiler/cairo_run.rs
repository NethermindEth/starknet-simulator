use anyhow::Error;
use cairo_lang_sierrra::program::Program as SierraProgram;

use cairo1_run::{
    cairo_run::cairo_run_program, Cairo1RunConfig, CairoRunner, MaybeRelocatable, VirtualMachine,
};

pub fn run(
    sierra_program: &SierraProgram,
    cairo_run_config: Cairo1RunConfig,
) -> Result<
    (
        CairoRunner,
        VirtualMachine,
        Vec<MaybeRelocatable>,
        Option<String>,
    ),
    Error,
> {
    let cairo_runner = cairo_run_program(sierra_program, cairo_run_config)?;
    Ok(cairo_runner)
}
