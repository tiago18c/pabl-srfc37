use pinocchio::pubkey::Pubkey;

use super::{Discriminator, Transmutable};


#[repr(C)]
pub struct ListConfig {
    pub discriminator: u8,
    pub authority: Pubkey,
    pub seed: Pubkey,
    pub wallets_count: u64,
    pub mode: u8,
}

impl ListConfig {
    pub const SEED_PREFIX: &'static [u8] = b"list_config";

    pub fn get_mode(&self) -> Mode {
        match self.mode {
            0 => Mode::Allow,
            1 => Mode::AllowAllEoas,
            _ => Mode::Block,
        }
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode as u8;
    }
}

impl Transmutable for ListConfig {
    const LEN: usize = 1 + 32 + 32 + 8 + 1;
}

impl Discriminator for ListConfig {
    const DISCRIMINATOR: u8 = 0x01;

    fn is_initialized(&self) -> bool {
        self.discriminator == Self::DISCRIMINATOR
    }
}

#[repr(u8)]
pub enum Mode {
    Allow,
    AllowAllEoas,
    Block
}