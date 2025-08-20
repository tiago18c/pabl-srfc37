use pinocchio::program_error::ProgramError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ABLError {
    InvalidInstruction,

    InvalidAuthority,
    AccountBlocked,
    NotEnoughAccounts,
    InvalidAccountData,
    UninitializedAccount,
    InvalidSystemProgram,
    InvalidConfigAccount,
    AccountNotWritable,
    InvalidMint,
    InvalidExtraMetasAccount,
    ImmutableOwnerExtensionMissing,
    InvalidData,
    InvalidEbaltsMintConfig,
}


impl From<ABLError> for ProgramError {
    fn from(e: ABLError) -> Self {
        ProgramError::Custom(e as u32)
    }
}