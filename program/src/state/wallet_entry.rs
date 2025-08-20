use pinocchio::pubkey::Pubkey;

use super::{Discriminator, Transmutable};

#[repr(C)]
pub struct WalletEntry {
    pub discriminator: u8,
    pub wallet_address: Pubkey,
    pub list_config: Pubkey,
}

impl WalletEntry {
    pub const SEED_PREFIX: &'static [u8] = b"wallet_entry";
}

impl Transmutable for WalletEntry {
    const LEN: usize = 1 + 32 + 32;
}

impl Discriminator for WalletEntry {
    const DISCRIMINATOR: u8 = 0x02;

    fn is_initialized(&self) -> bool {
        self.discriminator == Self::DISCRIMINATOR
    }
}
