use pinocchio::{account_info::AccountInfo, instruction::Signer, program_error::ProgramError, pubkey::find_program_address, seeds, sysvars::{rent::Rent, Sysvar}, ProgramResult};

use crate::{load, load_mut_unchecked, ABLError, ListConfig, Discriminator, Transmutable, WalletEntry};


pub struct AddWallet<'a> {
    pub authority: &'a AccountInfo,
    pub list_config: &'a AccountInfo,
    pub wallet: &'a AccountInfo,
    pub wallet_entry: &'a AccountInfo,
    pub system_program: &'a AccountInfo,
    pub wallet_entry_bump: u8,
}

impl<'a> AddWallet<'a> {
    pub const DISCRIMINATOR: u8 = 0x02;

    pub fn process(&self) -> ProgramResult {
        let lamports = Rent::get()?.minimum_balance(WalletEntry::LEN);

        let bump_seed = [self.wallet_entry_bump];
        let seeds = seeds!(WalletEntry::SEED_PREFIX, self.list_config.key(), self.wallet.key(), &bump_seed);
        let signer = Signer::from(&seeds);
            
        pinocchio_system::instructions::CreateAccount {
            from: self.authority,
            to: self.wallet_entry,
            lamports,
            space: WalletEntry::LEN as u64,
            owner: &crate::ID,
        }.invoke_signed(&[signer])?;

        let mut data = self.wallet_entry.try_borrow_mut_data()?;
        let wallet_entry = unsafe { 
            load_mut_unchecked::<WalletEntry>(&mut data)? 
        };
        wallet_entry.discriminator = WalletEntry::DISCRIMINATOR;
        wallet_entry.wallet_address = *self.wallet.key();
        wallet_entry.list_config = *self.list_config.key();

        let config = unsafe { load_mut_unchecked::<ListConfig>(self.list_config.borrow_mut_data_unchecked())? };
        config.wallets_count = config.wallets_count.checked_add(1).ok_or(ProgramError::ArithmeticOverflow)?;
        
        Ok(())
    }
}


impl<'a> TryFrom<&'a [AccountInfo]> for AddWallet<'a> {
    type Error = ABLError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [authority, list_config, wallet, wallet_entry, system_program] = accounts else {
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

        let (_, wallet_entry_bump) = find_program_address(&[WalletEntry::SEED_PREFIX, list_config.key(), wallet.key()], &crate::ID);

        // check if system program is valid
        if system_program.key().ne(&pinocchio_system::ID) {
            return Err(ABLError::InvalidSystemProgram);
        }

        Ok(Self {
            authority,
            list_config,
            wallet,
            wallet_entry,
            system_program,
            wallet_entry_bump,
        })
    }
}