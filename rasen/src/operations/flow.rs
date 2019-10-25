use rspirv::mr::{Instruction, Operand};
use spirv_headers::{LoopControl, MemoryAccess, Op, StorageClass};

use builder::Builder;
use errors::{ErrorKind, Result};
use module::FunctionRef;
use types::TypeName;

pub(crate) fn loop_(
    cond: FunctionRef,
    body: FunctionRef,
    module: &mut impl Builder,
    args: Vec<(&'static TypeName, u32)>,
) -> Result<(&'static TypeName, u32)> {
    if args.len() != 1 {
        bail!(ErrorKind::WrongArgumentsCount(args.len(), 1));
    }

    let (init_ty, init_id) = args[0];

    let (cond_id, cond_args, cond_res) = if let Some(res) = module.get_function(cond) {
        res
    } else {
        bail!(ErrorKind::MissingFunction(cond))
    };

    assert_eq!(cond_args, &[init_ty]);
    assert_eq!(
        cond_res,
        Some(TypeName::BOOL),
        "Condition function should return a boolean"
    );

    let (body_id, body_args, body_res) = if let Some(res) = module.get_function(body) {
        res
    } else {
        bail!(ErrorKind::MissingFunction(body))
    };

    assert_eq!(body_args, &[init_ty]);
    assert_eq!(body_res, Some(init_ty));

    let res_type = module.register_type(init_ty);
    let var_type = module.register_type(init_ty.as_ptr(false));
    let bool_type = module.register_type(TypeName::BOOL);

    let state_id = module.get_id();
    module.push_instruction(Instruction::new(
        Op::Variable,
        Some(var_type),
        Some(state_id),
        vec![Operand::StorageClass(StorageClass::Function)],
    ));

    module.push_instruction(Instruction::new(
        Op::Store,
        None,
        None,
        vec![
            Operand::IdRef(state_id),
            Operand::IdRef(init_id),
            Operand::MemoryAccess(MemoryAccess::empty()),
        ],
    ));
    let header_block = module.get_id();
    module.push_instruction(Instruction::new(
        Op::Branch,
        None,
        None,
        vec![Operand::IdRef(header_block)],
    ));

    // Header Block
    module.push_instruction(Instruction::new(
        Op::Label,
        None,
        Some(header_block),
        Vec::new(),
    ));

    let merge_block = module.get_id();
    let continue_block = module.get_id();

    module.push_instruction(Instruction::new(
        Op::LoopMerge,
        None,
        None,
        vec![
            Operand::IdRef(merge_block),
            Operand::IdRef(continue_block),
            Operand::LoopControl(LoopControl::NONE),
        ],
    ));

    let entry_block = module.get_id();
    module.push_instruction(Instruction::new(
        Op::Branch,
        None,
        None,
        vec![Operand::IdRef(entry_block)],
    ));

    // Entry Block
    module.push_instruction(Instruction::new(
        Op::Label,
        None,
        Some(entry_block),
        Vec::new(),
    ));

    let cond_load_id = module.get_id();
    module.push_instruction(Instruction::new(
        Op::Load,
        Some(res_type),
        Some(cond_load_id),
        vec![
            Operand::IdRef(state_id),
            Operand::MemoryAccess(MemoryAccess::empty()),
        ],
    ));

    let check_id = module.get_id();
    module.push_instruction(Instruction::new(
        Op::FunctionCall,
        Some(bool_type),
        Some(check_id),
        vec![Operand::IdRef(cond_id), Operand::IdRef(cond_load_id)],
    ));

    let body_block = module.get_id();
    module.push_instruction(Instruction::new(
        Op::BranchConditional,
        None,
        None,
        vec![
            Operand::IdRef(check_id),
            Operand::IdRef(body_block),
            Operand::IdRef(merge_block),
        ],
    ));

    // Body Block
    module.push_instruction(Instruction::new(
        Op::Label,
        None,
        Some(body_block),
        Vec::new(),
    ));

    let ret_id = module.get_id();
    module.push_instruction(Instruction::new(
        Op::FunctionCall,
        Some(res_type),
        Some(ret_id),
        vec![Operand::IdRef(body_id), Operand::IdRef(cond_load_id)],
    ));

    module.push_instruction(Instruction::new(
        Op::Store,
        None,
        None,
        vec![
            Operand::IdRef(state_id),
            Operand::IdRef(ret_id),
            Operand::MemoryAccess(MemoryAccess::empty()),
        ],
    ));

    module.push_instruction(Instruction::new(
        Op::Branch,
        None,
        None,
        vec![Operand::IdRef(continue_block)],
    ));

    // Continue Block
    module.push_instruction(Instruction::new(
        Op::Label,
        None,
        Some(continue_block),
        Vec::new(),
    ));

    module.push_instruction(Instruction::new(
        Op::Branch,
        None,
        None,
        vec![Operand::IdRef(header_block)],
    ));

    // Merge Block
    module.push_instruction(Instruction::new(
        Op::Label,
        None,
        Some(merge_block),
        Vec::new(),
    ));

    let result_id = module.get_id();
    module.push_instruction(Instruction::new(
        Op::Load,
        Some(res_type),
        Some(result_id),
        vec![
            Operand::IdRef(state_id),
            Operand::MemoryAccess(MemoryAccess::empty()),
        ],
    ));

    Ok((init_ty, result_id))
}
