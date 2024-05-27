use anyhow::Context;
use cairo_lang_casm::assembler::{ApUpdate, FpUpdate, Op1Addr, Opcode, PcUpdate, Res};
use cairo_lang_casm::operand::Register;
use cairo_lang_sierra::ProgramParser;
use cairo_lang_sierra_to_casm::compiler::{compile, CairoProgram};
use cairo_lang_sierra_to_casm::metadata::calc_metadata;
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};

use indexmap::IndexMap;
use std::fs;

/// Cairo instruction structure flags.
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Op1AddrI {
    Imm,
    AP,
    FP,
    Op0,
}
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ResI {
    Op1,
    Add,
    Mul,
    Unconstrained,
}
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PcUpdateI {
    Regular,
    Jump,
    JumpRel,
    Jnz,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ApUpdateI {
    Regular,
    Add,
    Add1,
    Add2,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum FpUpdateI {
    Regular,
    ApPlus2,
    Dst,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum OpcodeI {
    Nop,
    AssertEq,
    Call,
    Ret,
}

/// The low level representation of a cairo instruction.
#[allow(dead_code)]
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InstructionRepr {
    pub off0: i16,
    pub off1: i16,
    pub off2: i16,
    pub imm: Option<BigInt>,
    pub dst_register: Register,
    pub op0_register: Register,
    pub op1_addr: Op1AddrI,
    pub res: ResI,
    pub pc_update: PcUpdateI,
    pub ap_update: ApUpdateI,
    pub fp_update: FpUpdateI,
    pub opcode: OpcodeI,
}

pub type CasmSierraMapping = IndexMap<u64, Vec<u64>>;
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CasmInstruction {
    pub memory: String,
    pub instruction_index: usize,
    pub instruction_representation: Option<InstructionRepr>,
}
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CasmSierraMappingInstruction {
    pub casm_instructions: Vec<CasmInstruction>,
    pub casm_sierra_mapping: CasmSierraMapping,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SierraCompile {
    pub casm_sierra_mapping_instruction: CasmSierraMappingInstruction,
    pub casm: String,
}

pub fn compile_sierra_to_casm(path: String) -> Result<SierraCompile, anyhow::Error> {
    let sierra_program = fs::read_to_string(path).expect("Could not read file!");
    let program = ProgramParser::new()
        .parse(&sierra_program)
        .map_err(|_| anyhow::anyhow!("Failed to parse sierra program"))?;

    let cairo_program = compile(
        &program,
        &calc_metadata(&program, Default::default())
            .with_context(|| "Failed calculating Sierra variables.")?,
        true,
    )
    .with_context(|| "Compilation failed.")?;

    if let Ok(casm_sierra_mapping_instruction) =
        get_casm_sierra_mapping_instructions(cairo_program.clone())
    {
        Ok(SierraCompile {
            casm_sierra_mapping_instruction,
            casm: cairo_program.to_string(),
        })
    } else {
        Err(anyhow::anyhow!("Failed to compile sierra to casm"))
    }
}

pub fn get_casm_sierra_mapping_instructions(
    cairo_program: CairoProgram,
) -> Result<CasmSierraMappingInstruction, anyhow::Error> {
    let instructions = cairo_program.instructions;
    let mut casm_instructions = Vec::new();
    for (index, instruction) in instructions.iter().enumerate() {
        let instruction_representation = instruction.assemble();
        let mut first = true;
        let encoded_instructions = instruction_representation.encode();
        for encoded_instruction in encoded_instructions.iter() {
            let hex_instruction = format!("0x{:x}", encoded_instruction);
            let instruction_representation = instruction.assemble();

            let op1_addr = match instruction_representation.op1_addr {
                Op1Addr::Imm => Op1AddrI::Imm,
                Op1Addr::AP => Op1AddrI::AP,
                Op1Addr::FP => Op1AddrI::FP,
                Op1Addr::Op0 => Op1AddrI::Op0,
            };

            let res = match instruction_representation.res {
                Res::Op1 => ResI::Op1,
                Res::Add => ResI::Add,
                Res::Mul => ResI::Mul,
                Res::Unconstrained => ResI::Unconstrained,
            };

            let pc_update = match instruction_representation.pc_update {
                PcUpdate::Regular => PcUpdateI::Regular,
                PcUpdate::Jump => PcUpdateI::Jump,
                PcUpdate::JumpRel => PcUpdateI::JumpRel,
                PcUpdate::Jnz => PcUpdateI::Jnz,
            };

            let ap_update = match instruction_representation.ap_update {
                ApUpdate::Regular => ApUpdateI::Regular,
                ApUpdate::Add => ApUpdateI::Add,
                ApUpdate::Add1 => ApUpdateI::Add1,
                ApUpdate::Add2 => ApUpdateI::Add2,
            };

            let fp_update = match instruction_representation.fp_update {
                FpUpdate::Regular => FpUpdateI::Regular,
                FpUpdate::ApPlus2 => FpUpdateI::ApPlus2,
                FpUpdate::Dst => FpUpdateI::Dst,
            };

            let opcode = match instruction_representation.opcode {
                Opcode::Nop => OpcodeI::Nop,
                Opcode::AssertEq => OpcodeI::AssertEq,
                Opcode::Call => OpcodeI::Call,
                Opcode::Ret => OpcodeI::Ret,
            };
            if first {
                casm_instructions.push(CasmInstruction {
                    memory: hex_instruction,
                    instruction_representation: Some(InstructionRepr {
                        off0: instruction_representation.off0,
                        off1: instruction_representation.off1,
                        off2: instruction_representation.off2,
                        imm: instruction_representation.imm,
                        dst_register: instruction_representation.dst_register,
                        op0_register: instruction_representation.op0_register,
                        op1_addr: op1_addr,
                        res: res,
                        pc_update: pc_update,
                        ap_update: ap_update,
                        fp_update: fp_update,
                        opcode: opcode,
                    }),
                    instruction_index: index,
                });
                first = false; // Set first to false after the first iteration
            } else {
                casm_instructions.push(CasmInstruction {
                    memory: hex_instruction,
                    instruction_representation: None,
                    instruction_index: index,
                });
            }
        }
    }

    let debug_info = cairo_program.debug_info;
    let sierra_statement_info = debug_info.sierra_statement_info;

    let mut casm_sierra_mapping = IndexMap::new();
    let mut sierra_statement_index = 0;
    for sierra_statement_debug_info in sierra_statement_info.iter() {
        let casm_instruction_index = sierra_statement_debug_info.instruction_idx;
        casm_sierra_mapping
            .entry(casm_instruction_index as u64)
            .or_insert_with(Vec::new)
            .push(sierra_statement_index);
        sierra_statement_index += 1;
    }

    Ok(CasmSierraMappingInstruction {
        casm_instructions,
        casm_sierra_mapping,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_sierra_to_casm() {
        let path = "Sierra_file_path.sierra".to_string();
        let casm_sierra_mapping = compile_sierra_to_casm(path).expect("Compilation failed");
        // println!("{:?}", casm_sierra_mapping);
    }
}
