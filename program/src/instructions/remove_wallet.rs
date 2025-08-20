use pinocchio::{account_info::AccountInfo, program_error::ProgramError, ProgramResult};

use crate::{load, load_mut_unchecked, ABLError, ListConfig, WalletEntry};


pub struct RemoveWallet<'a> {
    pub authority: &'a AccountInfo,
    pub list_config: &'a AccountInfo,
    pub wallet_entry: &'a AccountInfo,
}

impl<'a> RemoveWallet<'a> {
    pub const DISCRIMINATOR: u8 = 0x03;

    pub fn process(&self) -> ProgramResult {
        
        let destination_lamports = self.authority.lamports();

        unsafe {
            *self.authority.borrow_mut_lamports_unchecked() = destination_lamports
                .checked_add(self.wallet_entry.lamports())
                .ok_or(ProgramError::ArithmeticOverflow)?;
            self.wallet_entry.close_unchecked();
        }
        
        let config = unsafe { load_mut_unchecked::<ListConfig>(self.list_config.borrow_mut_data_unchecked())? };
        config.wallets_count = config.wallets_count.checked_sub(1).ok_or(ProgramError::ArithmeticOverflow)?;

        self.wallet_entry.resize(0)?;

        Ok(())
    }
}

impl<'a> TryFrom<&'a [AccountInfo]> for RemoveWallet<'a> {
    type Error = ABLError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [authority, list_config, wallet_entry] = accounts else {
            return Err(ABLError::NotEnoughAccounts);
        };

        if !list_config.is_owned_by(&crate::ID) {
            return Err(ABLError::InvalidConfigAccount);
        }
        
        let cfg = unsafe { load::<ListConfig>(list_config.borrow_data_unchecked())? };
        
        if !authority.is_signer() || cfg.authority.ne(authority.key()) {
            return Err(ABLError::InvalidAuthority);
        }
        
        if !list_config.is_writable() && !wallet_entry.is_writable() {
            return Err(ABLError::AccountNotWritable);
        }

        if unsafe { load::<WalletEntry>(wallet_entry.borrow_data_unchecked()).is_err() }{
            return Err(ABLError::InvalidAccountData);
        }

        Ok(Self {
            authority,
            list_config,
            wallet_entry,
        })
    }
}