use cairo_lang_starknet_classes_2_point_6::casm_contract_class::CasmContractClass;
use cairo_vm::{
    hint_processor::cairo_1_hint_processor::hint_processor::Cairo1HintProcessor,
    types::{builtin_name::BuiltinName, layout_name::LayoutName, relocatable::MaybeRelocatable},
    vm::{
        errors::{cairo_run_errors::CairoRunError, vm_exception::get_traceback},
        runners::cairo_runner::{CairoArg, CairoRunner, RunResources},
        trace::trace_entry::RelocatedTraceEntry,
        vm_core::VirtualMachine,
    },
};
use hex::decode;
use serde::{Deserialize, Serialize};
use starknet_types_core::felt::Felt as Felt252;

fn hex_to_string(hex: &str) -> Result<String, hex::FromHexError> {
    //remove the 0x prefix
    let hex = &hex[2..];
    let bytes = decode(hex)?;
    let string = String::from_utf8(bytes).expect("Invalid UTF-8 sequence");
    Ok(string)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContractExecutionResult {
    pub retdata: String,
    pub trace: Vec<RelocatedTraceEntry>,
}

pub fn trace_error(
    casm_contract_class: CasmContractClass,
    entrypoint_offset: usize,
    args: &[MaybeRelocatable],
) -> Result<ContractExecutionResult, CairoRunError> {
    let mut hint_processor =
        Cairo1HintProcessor::new(&casm_contract_class.hints, RunResources::default());

    let mut runner = CairoRunner::new(
        &(casm_contract_class.clone().try_into().unwrap()),
        LayoutName::all_cairo,
        false,
    )
    .unwrap();
    let mut vm = VirtualMachine::new(true);

    let program_builtins = get_casm_contract_builtins(&casm_contract_class, entrypoint_offset);
    runner
        .initialize_function_runner_cairo_1(&mut vm, &program_builtins)
        .unwrap();

    // Implicit Args
    let syscall_segment = MaybeRelocatable::from(vm.add_memory_segment());

    let builtins = runner.get_program_builtins();

    let builtin_segment: Vec<MaybeRelocatable> = vm
        .get_builtin_runners()
        .iter()
        .filter(|b| builtins.contains(&b.name()))
        .flat_map(|b| b.initial_stack())
        .collect();

    let initial_gas = MaybeRelocatable::from(usize::MAX);

    let mut implicit_args = builtin_segment;
    implicit_args.extend([initial_gas]);
    implicit_args.extend([syscall_segment]);

    // Load builtin costs
    let builtin_costs: Vec<MaybeRelocatable> =
        vec![0.into(), 0.into(), 0.into(), 0.into(), 0.into()];
    let builtin_costs_ptr = vm.add_memory_segment();
    vm.load_data(builtin_costs_ptr, &builtin_costs).unwrap();

    // Load extra data
    let core_program_end_ptr =
        (runner.program_base.unwrap() + runner.get_program().data_len()).unwrap();
    let program_extra_data: Vec<MaybeRelocatable> =
        vec![0x208B7FFF7FFF7FFE.into(), builtin_costs_ptr.into()];
    vm.load_data(core_program_end_ptr, &program_extra_data)
        .unwrap();

    // Load calldata
    let calldata_start = vm.add_memory_segment();
    let calldata_end = vm.load_data(calldata_start, &args.to_vec()).unwrap();

    // Create entrypoint_args
    let mut entrypoint_args: Vec<CairoArg> = implicit_args
        .iter()
        .map(|m| CairoArg::from(m.clone()))
        .collect();
    entrypoint_args.extend([
        MaybeRelocatable::from(calldata_start).into(),
        MaybeRelocatable::from(calldata_end).into(),
    ]);
    let entrypoint_args: Vec<&CairoArg> = entrypoint_args.iter().collect();

    // Run contract entrypoint
    match runner.run_from_entrypoint(
        entrypoint_offset,
        &entrypoint_args,
        true,
        Some(runner.get_program().data_len() + program_extra_data.len()),
        &mut vm,
        &mut hint_processor,
    ) {
        Ok(_) => {
            println!("Execution completed successfully.");
        }
        Err(e) => {
            let traceback = get_traceback(&vm, &runner);
            println!("Error during execution: {:?}", e);
            println!("Traceback: {:?}", traceback);
            return Err(e);
        }
    }

    let program_segment_size = runner.get_program().data_len() + program_extra_data.len();
    let _ = runner.relocate_trace(&mut vm, &vec![1, 1 + program_segment_size]);

    let return_values = vm.get_return_values(5).unwrap();
    let retdata_start = return_values[3].get_relocatable().unwrap();
    let retdata_end = return_values[4].get_relocatable().unwrap();
    let vec_retdata: Vec<Felt252> = vm
        .get_integer_range(retdata_start, (retdata_end - retdata_start).unwrap())
        .unwrap()
        .iter()
        .map(|c| c.clone().into_owned())
        .collect();
    let hex_retdata: Vec<String> = vec_retdata.iter().map(|c| c.to_hex_string()).collect();
    let retdata = hex_to_string(&hex_retdata.join("")).unwrap();

    Ok(ContractExecutionResult {
        retdata,
        trace: runner.relocated_trace.unwrap(),
    })
}

fn get_casm_contract_builtins(
    contract_class: &CasmContractClass,
    entrypoint_offset: usize,
) -> Vec<BuiltinName> {
    contract_class
        .entry_points_by_type
        .external
        .iter()
        .find(|e| e.offset == entrypoint_offset)
        .unwrap()
        .builtins
        .iter()
        .map(|s| BuiltinName::from_str(s).expect("Invalid builtin name"))
        .collect()
}
