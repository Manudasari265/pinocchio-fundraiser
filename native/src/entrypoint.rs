use crate::instruction::{self, FundraiserInsruction};

use pinocchio::{
    account_info::AccountInfo, nostd_panic_handler, no_allocator, program_entrypoint, program_error::ProgramError, pubkey::Pubkey, ProgramResult
};

// This is the entrypoint for the program.
program_entrypoint!(process_instruction);
//Do not allocate memory.
no_allocator!();
// Use the no_std panic handler.
nostd_panic_handler!();

#[inline(always)]
fn process_instruction (
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let (discriminator_variant, instruction_data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match FundraiserInsruction::try_from(discriminator_variant)? {
        FundraiserInsruction::Initialize => {
            instruction::initialize::process_initialize(accounts, instruction_data)
        }
        FundraiserInsruction::Contribute => {
            instruction::contribute::process_contribute(accounts, instruction_data)
        }
        FundraiserInsruction::CheckContribution => {
            instruction::check::process_check(accounts, instruction_data)
        }
        FundraiserInsruction::RefundAndClose => {
            instruction::refund::process_refund(accounts, instruction_data)
        }
    }
}