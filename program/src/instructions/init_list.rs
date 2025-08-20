use pinocchio::{account_info::AccountInfo, instruction::Signer, pubkey::find_program_address, seeds, sysvars::{rent::Rent, Sysvar}, ProgramResult};

use crate::{load_mut_unchecked, ABLError, ListConfig, Discriminator, Transmutable};



pub struct InitList<'a> {
    pub authority: &'a AccountInfo,
    pub config: &'a AccountInfo,
    pub system_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for InitList<'a> {
    type Error = ABLError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [authority, config, system_program] = accounts else {
            return Err(ABLError::NotEnoughAccounts);
        };

        // check if system program is valid
        if system_program.key().ne(&pinocchio_system::ID) {
            return Err(ABLError::InvalidSystemProgram);
        }

        Ok(Self {
            authority,
            config,
            system_program,
        })
    }
}

impl<'a> InitList<'a> {
    pub const DISCRIMINATOR: u8 = 0x01;

    pub fn process(&self, remaining_data: &[u8]) -> ProgramResult {
        let [mode, seed @ ..] = remaining_data else {
            return Err(ABLError::InvalidData.into());
        };
        
        if seed.len() != 32 {
            return Err(ABLError::InvalidData.into());
        }
        
        if *mode > 2u8 {
            return Err(ABLError::InvalidData.into());
        }
        
        let lamports = Rent::get()?.minimum_balance(ListConfig::LEN);
        
        // find canonical bump to prepare signer seeds for cpi
        let seed = TryInto::<&[u8; 32]>::try_into(seed).unwrap();
        let (_, config_bump) = find_program_address(&[ListConfig::SEED_PREFIX, seed], &crate::ID);
        
        // prepare signer seeds for cpi
        let bump_seed = [config_bump];
        let seeds = seeds!(ListConfig::SEED_PREFIX, seed, &bump_seed);
        let signer = Signer::from(&seeds);
            
        pinocchio_system::instructions::CreateAccount {
            from: self.authority,
            to: self.config,
            lamports,
            space: ListConfig::LEN as u64,
            owner: &crate::ID,
        }.invoke_signed(&[signer])?;

        let mut data = self.config.try_borrow_mut_data()?;
        let config = unsafe { 
            load_mut_unchecked::<ListConfig>(&mut data)? 
        };
        config.discriminator = ListConfig::DISCRIMINATOR;
        config.authority = *self.authority.key();
        config.seed = *seed;
        config.mode = *mode;

        Ok(())
    }
}