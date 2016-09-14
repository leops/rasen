//! Transform operations into bytecode

use std::mem;
use spirv::*;

/// Utility struct for building standard SPIR-V strings
#[derive(Default)]
pub struct StringBuilder {
    words: Vec<u32>,
    bytes: Vec<u8>,
}

impl StringBuilder {
    /// Turns a `String` into a padded list of 32-bits words
    pub fn to_words(string: String) -> Vec<u32> {
        let mut builder: StringBuilder = Default::default();

        for byte in string.as_bytes() {
            builder.push_byte(*byte);
        }

        builder.push_byte(0);
        while builder.bytes.len() != 0 {
            builder.push_byte(0);
        }

        builder.words
    }

    fn push_byte(&mut self, byte: u8) {
        self.bytes.push(byte);
        if self.bytes.len() == 4 {
            unsafe {
                let mut arr = [0u8; 4];
                for (place, element) in arr.iter_mut().zip(self.bytes.iter()) {
                    *place = *element;
                }

                self.words.push(mem::transmute::<[u8; 4], u32>(arr));
            }

            self.bytes.clear();
        }
    }
}

fn to_opcode(word_count: u16, op_id: u16) -> u32 {
    let input: [u16; 2] = [op_id, word_count];

    unsafe {
        mem::transmute::<[u16; 2], u32>(input)
    }
}

/// Transforms a list of operations into a bytecode blob
pub fn to_bytecode(operations: Vec<Operation>) -> Vec<u8> {
    let header: Vec<u32> = vec![0x07230203, 0x00010000, 0, 128, 0];

    header.into_iter()
        .chain(
            operations.into_iter()
                .flat_map(|op| match op {
                    Operation::OpExtInstImport(var_id, name) => {
                        let name_words = StringBuilder::to_words(name);
                        vec![to_opcode(2 + name_words.len() as u16, 11), var_id].into_iter()
                            .chain(name_words.into_iter())
                            .collect()
                    },
                    Operation::OpExtInst(res_id, res_type, ext_id, func_id, args) =>
                        vec![to_opcode(5 + args.len() as u16, 12), res_type, res_id, ext_id, func_id].into_iter()
                            .chain(args.into_iter())
                            .collect(),
                    Operation::OpMemoryModel(addressing, memory) => vec![to_opcode(3, 14), addressing as u32, memory as u32],
                    Operation::OpEntryPoint(model, entry_id, name, args) => {
                        let name_words = StringBuilder::to_words(name);
                        vec![to_opcode(3 + (name_words.len() + args.len()) as u16, 15), model as u32, entry_id].into_iter()
                            .chain(name_words.into_iter())
                            .chain(args.into_iter())
                            .collect()
                    },
                    Operation::OpExecutionMode(entry_id, mode) => vec![to_opcode(3, 16), entry_id, mode as u32],
                    Operation::OpCapability(capability) => vec![to_opcode(2, 17), capability as u32],
                    Operation::OpTypeVoid(id) => vec![to_opcode(2, 19), id],
                    Operation::OpTypeFloat(id, size) => vec![to_opcode(3, 22), id, size],
                    Operation::OpTypeVector(id, type_id, size) => vec![to_opcode(4, 23), id, type_id, size],
                    Operation::OpTypePointer(ptr_type, storage_class, type_id) => vec![to_opcode(4, 32), ptr_type, storage_class as u32, type_id],
                    Operation::OpTypeFunction(id, ret_type) => vec![to_opcode(3, 33), id, ret_type],
                    Operation::OpConstant(res_id, const_type, const_value) => vec![to_opcode(4, 43), const_type, res_id, const_value],
                    Operation::OpConstantComposite(res_id, type_id, values) => {
                        vec![to_opcode(3 + values.len() as u16, 44), type_id, res_id].into_iter()
                            .chain(values.into_iter())
                            .collect()
                    },
                    Operation::OpFunction(res_id, type_id, func_control, func_type) => vec![to_opcode(5, 54), type_id, res_id, func_control.into(), func_type],
                    Operation::OpVariable(var_id, ptr_type, storage_class) => vec![to_opcode(4, 59), ptr_type, var_id, storage_class as u32],
                    Operation::OpLoad(res_id, type_id, var_id) => vec![to_opcode(4, 61), type_id, res_id, var_id],
                    Operation::OpStore(var_id, value_id) => vec![to_opcode(3, 62), var_id, value_id],
                    Operation::OpDecorate(var_id, decoration, value) => vec![to_opcode(4, 71), var_id, decoration as u32, value],
                    Operation::OpVectorTimesScalar(res_id, vec_type, vec_id, scalar_id) => vec![to_opcode(5, 142), vec_type, res_id, vec_id, scalar_id],
                    Operation::OpDot(res_id, res_type, arg_0, arg_1) => vec![to_opcode(5, 148), res_type, res_id, arg_0, arg_1],
                    Operation::OpLabel(res_id) => vec![to_opcode(2, 248), res_id],
                    Operation::OpReturn => vec![to_opcode(1, 253)],
                    Operation::OpFunctionEnd => vec![to_opcode(1, 56)],
                })
        )
        .flat_map(|words| unsafe {
            let as_bytes = mem::transmute::<u32, [u8; 4]>(words);
            vec![as_bytes[0], as_bytes[1], as_bytes[2], as_bytes[3]]
        })
        .collect()
}
