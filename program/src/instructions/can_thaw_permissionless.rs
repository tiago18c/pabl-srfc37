use pinocchio::{account_info::AccountInfo, ProgramResult};

use crate::{load, ABLError, ListConfig, WalletEntry};

use solana_curve25519::edwards::PodEdwardsPoint;

///
/// SECURITY ASSUMPTIONS OVER TX-HOOK
///
/// 1- its called by the token-2022 program
/// 2- if some other program is calling it, we don't care as we don't write state here
/// 2- its inputs are already sanitized by the token-2022 program
/// 3- if some other program is calling it with invalid inputs, we don't care as we only read state and return ok/nok
/// 4- there may be 3 different extra metas setup
/// 4.1- no extra accounts
/// 4.2- only source wallet block
/// 4.3- both source and destination wallet blocks
/// 5- given all the above we can skip a lot of type and owner checks

pub struct CanThawPermissionless<'a> {
    pub authority: &'a AccountInfo,
    pub token_account: &'a AccountInfo,
    pub mint: &'a AccountInfo,
    pub owner: &'a AccountInfo,
    pub extra_metas: &'a AccountInfo,
    pub remaining_accounts: &'a [AccountInfo],
}

impl<'a> CanThawPermissionless<'a> {
    pub const DISCRIMINATOR: u8 = 0x8;

    pub fn process(&self) -> ProgramResult {
        // remaining accounts should be pairs of list and ab_wallet
        let mut remaining_accounts = self.remaining_accounts.iter();
        while let Some(list) = remaining_accounts.next() {
            let ab_wallet = remaining_accounts.next().unwrap();
            
            CanThawPermissionless::validate_thaw_list(list, self.owner,ab_wallet).map_err(|e| {
                pinocchio_log::log!("Failed to pass validation for list {}", list.key());
                e
            })?;
        }

        Ok(())
    }

    fn validate_thaw_list(list: &AccountInfo, owner: &AccountInfo, wallet_entry: &AccountInfo) -> ProgramResult {
        let list_data: &[u8] = &list.try_borrow_data()?;
        let list_config = unsafe { load::<ListConfig>(list_data)? };

        // 3 operation modes
        // allow: only wallets that have been allowlisted can thaw, requires previously created ABWallet account
        // block: only wallets that have been blocklisted can't thaw, thawing requires ABWallet to not exist
        // allow with permissionless eoas: all wallets that can sign can thaw, otherwise requires previously created ABWallet account (for PDAs)
        match list_config.get_mode() {
            crate::Mode::Allow => {
                let ab_wallet_data: &[u8] = &wallet_entry.try_borrow_data()?;
                let _ = unsafe {
                    load::<WalletEntry>(ab_wallet_data).map_err(|_| ABLError::AccountBlocked)?
                };

                Ok(())
            }
            crate::Mode::AllowAllEoas => {
                let pt = PodEdwardsPoint(owner.key().clone());

                if !solana_curve25519::edwards::validate_edwards(&pt) {
                    let ab_wallet_data: &[u8] = &wallet_entry.try_borrow_data()?;
                    let _ = unsafe {
                        load::<WalletEntry>(ab_wallet_data).map_err(|_| ABLError::AccountBlocked)?
                    };
                }

                Ok(())
            }
            crate::Mode::Block => {
                let ab_wallet_data: &[u8] = &wallet_entry.try_borrow_data()?;
                let res = unsafe { load::<WalletEntry>(ab_wallet_data) };

                if res.is_ok() {
                    Err(ABLError::AccountBlocked.into())
                } else {
                    Ok(())
                }
            }
        }
    }
}

impl<'a> TryFrom<&'a [AccountInfo]> for CanThawPermissionless<'a> {
    type Error = ABLError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        /*
        TX HOOK GETS CALLED WITH:
         1- authority
         2- token account
         3- mint
         4- owner
         5- extra account metas
         6- (optional) source wallet block
         7- (optional) destination wallet block
         */

        let [authority, token_account, mint, owner, extra_metas, remaining_accounts @ ..] =
            accounts
        else {
            return Err(ABLError::NotEnoughAccounts);
        };

        Ok(Self {
            authority,
            token_account,
            mint,
            owner,
            extra_metas,
            remaining_accounts,
        })
    }
}
