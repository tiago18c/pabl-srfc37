#![no_std]
 
use pinocchio::{account_info::AccountInfo, no_allocator, program_entrypoint, program_error::ProgramError, pubkey::Pubkey, ProgramResult};
use pinocchio_pubkey::declare_id;
 
program_entrypoint!(process_instruction);
// Do not allocate memory.
no_allocator!();
// Use the no_std panic handler.
//nostd_panic_handler!();
 
pub mod instructions;
pub use instructions::*;
pub mod error;
pub use error::*;
pub mod state;
pub use state::*;

declare_id!("BLoCKLSG2qMQ9YxEyrrKKAQzthvW4Lu8Eyv74axF6mf");

 
#[inline(always)]
fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let [disc, remaining_data @ ..] = instruction_data else {
        return Err(ABLError::InvalidInstruction.into());
    };
    
    
    match *disc {
        CanThawPermissionless::DISCRIMINATOR => CanThawPermissionless::try_from(accounts)?.process(),
        InitList::DISCRIMINATOR => InitList::try_from(accounts)?.process(remaining_data),
        AddWallet::DISCRIMINATOR => AddWallet::try_from(accounts)?.process(),
        RemoveWallet::DISCRIMINATOR => RemoveWallet::try_from(accounts)?.process(),
        SetupExtraMetas::DISCRIMINATOR => SetupExtraMetas::try_from(accounts)?.process(),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}