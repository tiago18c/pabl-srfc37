pub mod program_test;
use allow_block_list_client::types::Mode;
use solana_sdk::signer::Signer;

use crate::program_test::TestContext;

#[tokio::test]
async fn fails_to_thaw_non_eoa_wallet() {
    let mut context = TestContext::new();

    let _ = context.setup_ebalts();
    let list_config = context.create_list(Mode::AllowAllEoas);
    let _ = context.setup_extra_metas(&[list_config]);

    let ta = context.create_token_account_from_pubkey(&list_config);

    let res = context.thaw_permissionless(&list_config, &ta).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn thaws_non_eoa_added_owner() {
    let mut context = TestContext::new();

    let _ = context.setup_ebalts();
    let list_config = context.create_list(Mode::AllowAllEoas);
    let _ = context.setup_extra_metas(&[list_config]);

    // list_config is acting as the ta owner for test simplicity
    // as this is one of the off-the-curve available pubkeys
    let _ = context.add_wallet_to_list(&list_config, &list_config);

    let ta = context.create_token_account_from_pubkey(&list_config);

    let res = context.thaw_permissionless(&list_config, &ta).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn thaws_eoa_wallet() {
    let mut context = TestContext::new();

    let _ = context.setup_ebalts();
    let list_config = context.create_list(Mode::AllowAllEoas);
    let _ = context.setup_extra_metas(&[list_config]);

    let wallet = solana_keypair::Keypair::new();
    let ta = context.create_token_account(&wallet);

    let res = context.thaw_permissionless(&wallet.pubkey(), &ta).await;
    assert!(res.is_ok());
}
