use inkwell::context::Context;
use inkwell::module::Module;

use bfir::{AstNode, Cell};
use execution::ExecutionState;

pub fn compile_to_module(
    context: &mut Context,
    module_name: &str,
    target_triple: Option<String>,
    instr: &[AstNode],
    initial_state: &ExecutionState,
) -> Module {
    let mut module = create_module(context, module_name, target_triple);
    module
}

fn create_module(context: &Context, module_name: &str, target_triple: Option<String>) -> Module {
    let module = context.create_module(module_name);
    let triple = match target_triple {
        Some(target_triple) => inkwell::targets::TargetTriple::create(target_triple),
        None => inkwell::targets::TargetTriple::create("x86_64-pc-linux-gnu"),
    };
    module.set_triple(&triple);
    module
}
