//! Definitions for various SPIR-V values
//!
//! This module is (mostly) auto-generated from the enum definitions found at
//! https://www.khronos.org/registry/spir-v/api/1.1/spirv.json
//!
//! The enums defined as `Value` are retranscribed as simple enums, and the types defined as `Bit`
//! are represented with structs (having a named bool field for each bit). All these types can be
//! transformed into SPIR-V words with the `Into<u32>` trait.

include!{concat!(env!("OUT_DIR"), "/spirv.rs")}

#[derive(Clone)]
pub enum Operation {
    OpCapability(Capability),
    OpExtInstImport(u32, String),
    OpMemoryModel(AddressingModel, MemoryModel),
    OpExecutionMode(u32, ExecutionMode),
    OpEntryPoint(ExecutionModel, u32, String, Vec<u32>),
    OpDecorate(u32, Decoration, u32),
    OpTypeVoid(u32),
    OpTypeFunction(u32, u32),
    OpTypeFloat(u32, u32),
    OpTypeVector(u32, u32, u32),
    OpTypePointer(u32, StorageClass, u32),
    OpVariable(u32, u32, StorageClass),
    OpConstant(u32, u32, u32),
    OpConstantComposite(u32, u32, Vec<u32>),
    OpFunction(u32, u32, FunctionControl, u32),
    OpLabel(u32),
    OpLoad(u32, u32, u32),
    OpDot(u32, u32, u32, u32),
    OpExtInst(u32, u32, u32, u32, Vec<u32>),
    OpVectorTimesScalar(u32, u32, u32, u32),
    OpStore(u32, u32),
    OpReturn,
    OpFunctionEnd,
}
