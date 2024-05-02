pub mod cairo_sierra;
pub mod casm_sierra;
pub mod compiler;

fn main() {
    // run compiler
    // compiler::compile_contract::main();
    compiler::compile::main();
}
