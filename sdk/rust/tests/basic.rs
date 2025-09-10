pub mod program_test;
use allow_block_list_client::{
    accounts::{ListConfig, WalletEntry},
    types::Mode,
};
use solana_instruction::AccountMeta;
use solana_pubkey::Pubkey;
use solana_sdk::{signer::Signer, transaction::Transaction};

use crate::program_test::TestContext;

#[tokio::test]
async fn creates_list() {
    let mut context = TestContext::new();

    let seed = Pubkey::new_unique();
    let (list_config_address, _) =
        allow_block_list_client::accounts::ListConfig::find_pda(&context.auth.pubkey(), &seed);

    let ix = allow_block_list_client::instructions::CreateListBuilder::new()
        .authority(context.auth.pubkey())
        .list_config(list_config_address)
        .mode(Mode::Allow)
        .seed(seed)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.auth.pubkey()),
        &[context.auth.insecure_clone()],
        context.vm.latest_blockhash(),
    );
    let res = context.vm.send_transaction(tx);
    assert!(res.is_ok());

    let list_config = context.vm.get_account(&list_config_address).unwrap();
    let config = ListConfig::from_bytes(&list_config.data).unwrap();

    assert_eq!(config.authority, context.auth.pubkey());
    assert_eq!(config.seed, seed);
    assert_eq!(config.mode, Mode::Allow as u8);
    assert_eq!(config.wallets_count, 0);
}

#[tokio::test]
async fn creates_list_with_different_mode() {
    let mut context = TestContext::new();

    let list_config_address = context.create_list(Mode::Block);

    let list_config = context.vm.get_account(&list_config_address).unwrap();
    let config = ListConfig::from_bytes(&list_config.data).unwrap();

    assert_eq!(config.authority, context.auth.pubkey());
    assert_eq!(config.mode, Mode::Block as u8);

    let list_config_address = context.create_list(Mode::AllowAllEoas);

    let list_config = context.vm.get_account(&list_config_address).unwrap();
    let config = ListConfig::from_bytes(&list_config.data).unwrap();

    assert_eq!(config.authority, context.auth.pubkey());
    assert_eq!(config.mode, Mode::AllowAllEoas as u8);
}

#[tokio::test]
async fn deletes_list() {
    let mut context = TestContext::new();

    let list_config_address = context.create_list(Mode::Allow);

    let ix = allow_block_list_client::instructions::DeleteListBuilder::new()
        .authority(context.auth.pubkey())
        .list_config(list_config_address)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.auth.pubkey()),
        &[context.auth.insecure_clone()],
        context.vm.latest_blockhash(),
    );

    let res = context.vm.send_transaction(tx);
    assert!(res.is_ok());

    let list_config = context.vm.get_account(&list_config_address).unwrap();

    assert_eq!(list_config.data.len(), 0);
    assert_eq!(list_config.lamports, 0);
}

#[tokio::test]
async fn adds_wallet() {
    let mut context = TestContext::new();

    let wallet_address = Pubkey::new_unique();
    let list_config_address = context.create_list(Mode::Allow);
    let (wallet_entry, _) = allow_block_list_client::accounts::WalletEntry::find_pda(
        &list_config_address,
        &wallet_address,
    );

    let ix = allow_block_list_client::instructions::AddWalletBuilder::new()
        .authority(context.auth.pubkey())
        .list_config(list_config_address)
        .wallet(wallet_address)
        .wallet_entry(wallet_entry)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.auth.pubkey()),
        &[context.auth.insecure_clone()],
        context.vm.latest_blockhash(),
    );

    let res = context.vm.send_transaction(tx);
    assert!(res.is_ok());

    let list_config = context.vm.get_account(&list_config_address).unwrap();
    let config = ListConfig::from_bytes(&list_config.data).unwrap();

    assert_eq!(config.wallets_count, 1);

    let wallet_entry = context.vm.get_account(&wallet_entry).unwrap();
    let entry = WalletEntry::from_bytes(&wallet_entry.data).unwrap();

    assert_eq!(entry.wallet_address, wallet_address);
    assert_eq!(entry.list_config, list_config_address);
}

#[tokio::test]
async fn removes_wallet() {
    let mut context = TestContext::new();

    let wallet_address = Pubkey::new_unique();
    let list_config_address = context.create_list(Mode::Allow);
    let wallet_entry = context.add_wallet_to_list(&list_config_address, &wallet_address);

    let ix = allow_block_list_client::instructions::RemoveWalletBuilder::new()
        .authority(context.auth.pubkey())
        .list_config(list_config_address)
        .wallet_entry(wallet_entry)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.auth.pubkey()),
        &[context.auth.insecure_clone()],
        context.vm.latest_blockhash(),
    );

    let res = context.vm.send_transaction(tx);
    assert!(res.is_ok());

    let list_config = context.vm.get_account(&list_config_address).unwrap();
    let config = ListConfig::from_bytes(&list_config.data).unwrap();

    assert_eq!(config.wallets_count, 0);

    let wallet_entry = context.vm.get_account(&wallet_entry).unwrap();

    assert_eq!(wallet_entry.data.len(), 0);
    assert_eq!(wallet_entry.lamports, 0);
}

#[tokio::test]
async fn setup_list_extra_metas() {
    let mut context = TestContext::new();

    let mint_config = context.setup_token_acl();

    let list_config_address = context.create_list(Mode::Allow);

    let extra_metas = token_acl_interface::get_thaw_extra_account_metas_address(
        &context.token.mint,
        &allow_block_list_client::programs::ABL_ID,
    );

    let ix = allow_block_list_client::instructions::SetupExtraMetasBuilder::new()
        .authority(context.token.auth.pubkey())
        .mint(context.token.mint)
        .extra_metas(extra_metas)
        .token_acl_mint_config(mint_config)
        .add_remaining_account(AccountMeta::new_readonly(list_config_address, false))
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.token.auth.pubkey()),
        &[context.token.auth.insecure_clone()],
        context.vm.latest_blockhash(),
    );

    let res = context.vm.send_transaction(tx);
    assert!(res.is_ok());

    let wallet = solana_keypair::Keypair::new();
    let user_pubkey = wallet.pubkey();
    let (wallet_entry, _) = allow_block_list_client::accounts::WalletEntry::find_pda(
        &list_config_address,
        &wallet.pubkey(),
    );
    let ta = context.create_token_account(&wallet);

    let ix = token_acl_client::create_thaw_permissionless_instruction_with_extra_metas(
        &user_pubkey,
        &ta,
        &context.token.mint,
        &mint_config,
        &spl_token_2022::ID,
        &user_pubkey,
        false,
        |pubkey| {
            let account = context.vm.get_account(&pubkey);
            async move {
                match account {
                    Some(account) => Ok(Some(account.data)),
                    None => Ok(None),
                }
            }
        },
    )
    .await
    .unwrap();

    let mut rev_iter = ix.accounts.iter().rev();

    assert_eq!(rev_iter.next().unwrap().pubkey, wallet_entry);
    assert_eq!(rev_iter.next().unwrap().pubkey, list_config_address);
    assert_eq!(rev_iter.next().unwrap().pubkey, extra_metas);
    assert!(rev_iter.any(|account| account.pubkey == allow_block_list_client::programs::ABL_ID));
}

#[tokio::test]
async fn setup_list_extra_metas_with_multiple_lists() {
    let mut context = TestContext::new();

    let mint_config = context.setup_token_acl();

    let list_config_address = context.create_list(Mode::Allow);
    let list_config_address_2 = context.create_list(Mode::Block);
    let list_config_address_3 = context.create_list(Mode::AllowAllEoas);

    let extra_metas = token_acl_interface::get_thaw_extra_account_metas_address(
        &context.token.mint,
        &allow_block_list_client::programs::ABL_ID,
    );

    let ix = allow_block_list_client::instructions::SetupExtraMetasBuilder::new()
        .authority(context.token.auth.pubkey())
        .mint(context.token.mint)
        .extra_metas(extra_metas)
        .token_acl_mint_config(mint_config)
        .add_remaining_accounts(&[
            AccountMeta::new_readonly(list_config_address, false),
            AccountMeta::new_readonly(list_config_address_2, false),
            AccountMeta::new_readonly(list_config_address_3, false),
        ])
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&context.token.auth.pubkey()),
        &[context.token.auth.insecure_clone()],
        context.vm.latest_blockhash(),
    );

    let res = context.vm.send_transaction(tx);
    assert!(res.is_ok());

    let wallet = solana_keypair::Keypair::new();
    let user_pubkey = wallet.pubkey();
    let (wallet_entry, _) = allow_block_list_client::accounts::WalletEntry::find_pda(
        &list_config_address,
        &wallet.pubkey(),
    );
    let (wallet_entry2, _) = allow_block_list_client::accounts::WalletEntry::find_pda(
        &list_config_address_2,
        &wallet.pubkey(),
    );
    let (wallet_entry3, _) = allow_block_list_client::accounts::WalletEntry::find_pda(
        &list_config_address_3,
        &wallet.pubkey(),
    );
    let ta = context.create_token_account(&wallet);

    let ix = token_acl_client::create_thaw_permissionless_instruction_with_extra_metas(
        &user_pubkey,
        &ta,
        &context.token.mint,
        &mint_config,
        &spl_token_2022::ID,
        &user_pubkey,
        false,
        |pubkey| {
            let account = context.vm.get_account(&pubkey);
            async move {
                match account {
                    Some(account) => Ok(Some(account.data)),
                    None => Ok(None),
                }
            }
        },
    )
    .await
    .unwrap();

    assert!(ix
        .accounts
        .iter()
        .any(|account| account.pubkey == list_config_address));
    assert!(ix
        .accounts
        .iter()
        .any(|account| account.pubkey == list_config_address_2));
    assert!(ix
        .accounts
        .iter()
        .any(|account| account.pubkey == list_config_address_3));
    assert!(ix
        .accounts
        .iter()
        .any(|account| account.pubkey == wallet_entry));
    assert!(ix
        .accounts
        .iter()
        .any(|account| account.pubkey == wallet_entry2));
    assert!(ix
        .accounts
        .iter()
        .any(|account| account.pubkey == wallet_entry3));
}

#[tokio::test]
async fn setup_list_extra_metas_multiple_times() {
    let mut context = TestContext::new();

    let _mint_config = context.setup_token_acl();

    let list_config_address = context.create_list(Mode::Allow);
    let list_config_address_2 = context.create_list(Mode::Block);
    let list_config_address_3 = context.create_list(Mode::AllowAllEoas);

    let _res = context.setup_extra_metas(&[list_config_address]);
    let _res = context.setup_extra_metas(&[
        list_config_address,
        list_config_address_2,
        list_config_address_3,
    ]);
    let _res = context.setup_extra_metas(&[list_config_address, list_config_address_2]);
    let _res = context.setup_extra_metas(&[]);
}
