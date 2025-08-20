pub mod list_config;
pub mod wallet_entry;
pub use list_config::*;
pub use wallet_entry::*;

use crate::ABLError;

pub trait Transmutable {
    const LEN: usize;
}

pub trait Discriminator {
    const DISCRIMINATOR: u8;

    fn is_initialized(&self) -> bool;
}

/// Return a reference for an initialized `T` from the given bytes.
///
/// # Safety
///
/// The caller must ensure that `bytes` contains a valid representation of `T`.
#[inline(always)]
pub unsafe fn load<T: Discriminator + Transmutable>(bytes: &[u8]) -> Result<&T, ABLError> {
    load_unchecked(bytes).and_then(|t: &T| {
        // checks if the data is initialized
        if t.is_initialized() {
            Ok(t)
        } else {
            Err(ABLError::InvalidAccountData)
        }
    })
}

/// Return a reference for an initialized `T` from the given bytes.
///
/// # Safety
///
/// The caller must ensure that `bytes` contains a valid representation of `T`.
#[inline(always)]
pub unsafe fn load_mut<T: Discriminator + Transmutable>(bytes: &mut [u8]) -> Result<&mut T, ABLError> {
    load_mut_unchecked(bytes).and_then(|t: &mut T| {
        // checks if the data is initialized
        if t.is_initialized() {
            Ok(t)
        } else {
            Err(ABLError::InvalidAccountData)
        }
    })
}

/// Return a `T` reference from the given bytes.
///
/// This function does not check if the data is initialized.
///
/// # Safety
///
/// The caller must ensure that `bytes` contains a valid representation of `T`.
#[inline(always)]
pub unsafe fn load_unchecked<T: Transmutable>(bytes: &[u8]) -> Result<&T, ABLError> {
    if bytes.len() != T::LEN {
        return Err(ABLError::InvalidAccountData);
    }
    Ok(&*(bytes.as_ptr() as *const T))
}

/// Return a mutable `T` reference from the given bytes.
///
/// This function does not check if the data is initialized.
///
/// # Safety
///
/// The caller must ensure that `bytes` contains a valid representation of `T`.
#[inline(always)]
pub unsafe fn load_mut_unchecked<T: Transmutable>(
    bytes: &mut [u8],
) -> Result<&mut T, ABLError> {
    if bytes.len() != T::LEN {
        return Err(ABLError::InvalidAccountData);
    }
    Ok(&mut *(bytes.as_mut_ptr() as *mut T))
}

