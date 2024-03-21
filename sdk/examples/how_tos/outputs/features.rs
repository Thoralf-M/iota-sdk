// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! In this example we will build NFT outputs with all possible features.
//!
//! Rename `.env.example` to `.env` first, then run the command:
//! ```sh
//! cargo run --release --all-features --example output_features
//! ```

use iota_sdk::{
    client::Client,
    types::block::{
        address::{AccountAddress, MultiAddress, WeightedAddress},
        output::{
            feature::{Irc27Metadata, Irc30Metadata, IssuerFeature, MetadataFeature, SenderFeature, TagFeature},
            unlock_condition::{
                AddressUnlockCondition, ExpirationUnlockCondition, ImmutableAccountAddressUnlockCondition,
                StorageDepositReturnUnlockCondition, TimelockUnlockCondition,
            },
            AccountOutputBuilder, BasicOutputBuilder, DelegationId, DelegationOutputBuilder, Feature,
            FoundryOutputBuilder, NftId, NftOutputBuilder, SimpleTokenScheme, TokenScheme, UnlockCondition,
        },
    },
    wallet::FilterOptions,
    Wallet,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let logger_output_config = fern_logger::LoggerOutputConfigBuilder::new()
        .name("wallet.log")
        .target_exclusions(&["h2", "hyper", "rustls"])
        .level_filter(log::LevelFilter::Debug);
    let config = fern_logger::LoggerConfig::build()
        .with_output(logger_output_config)
        .finish();
    fern_logger::logger_init(config).unwrap();

    let wallet = Wallet::builder()
        .with_storage_path(&"./cli/stardust-cli-wallet-db")
        .finish()
        .await?;
    // needs to be copied from cli dir to dir where this example runs
    wallet.set_stronghold_password("test".to_string()).await?;

    wallet.sync(None).await?;

    let unspent_basic_output = wallet
        .ledger()
        .await
        .filtered_unspent_outputs(FilterOptions {
            output_types: Some(vec![0]),
            ..Default::default()
        })
        .next()
        .unwrap()
        .clone();

    let account_output_data = wallet.ledger().await.accounts().next().unwrap().clone();
    let account_address = AccountAddress::from(
        account_output_data
            .output
            .as_account()
            .account_id_non_null(&account_output_data.output_id),
    );

    let protocol_parameters = wallet.client().get_protocol_parameters().await?;
    let storage_score_params = wallet.client().get_storage_score_parameters().await?;
    let issuance = wallet.client().get_issuance().await?;
    let latest_slot_commitment_id = issuance.latest_commitment.id();

    let outputs = vec![
        BasicOutputBuilder::new_with_minimum_amount(storage_score_params)
            .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                wallet.address().await,
            )))
            .finish_output()?,
        // Enable when https://github.com/iotaledger/inx-indexer/issues/179 is fixed
        // BasicOutputBuilder::new_with_minimum_amount(storage_score_params)
        //     .with_mana(unspent_basic_output.output.available_mana(
        //         &protocol_parameters,
        //         unspent_basic_output.output_id.transaction_id().slot_index(),
        //         wallet.client().get_slot_index().await?,
        //     )?)
        //     .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
        //         MultiAddress::new(
        //             [
        //                 WeightedAddress::new(wallet.address().await.into_inner(), 1)?,
        //                 WeightedAddress::new(account_address, 1)?,
        //             ],
        //             1,
        //         )?,
        //     )))
        //     .finish_output()?,
        BasicOutputBuilder::new_with_amount(1_000_000)
            .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(
                wallet.address().await,
            )))
            .add_feature(TagFeature::new("Hello, Alphanet!")?)
            .add_feature(MetadataFeature::new([("Hello".to_owned(), b"Alphanet!".to_vec())])?)
            .finish_output()?,
        BasicOutputBuilder::new_with_amount(1_000_000)
            .with_unlock_conditions([
                UnlockCondition::Address(AddressUnlockCondition::new(wallet.address().await)),
                UnlockCondition::Expiration(ExpirationUnlockCondition::new(
                    wallet.address().await,
                    wallet.client().get_slot_index().await? + 5000,
                )?),
                UnlockCondition::Timelock(TimelockUnlockCondition::new(
                    wallet.client().get_slot_index().await? + 100,
                )?),
                UnlockCondition::StorageDepositReturn(StorageDepositReturnUnlockCondition::new(
                    wallet.address().await,
                    500_000,
                )?),
            ])
            .add_feature(SenderFeature::new(wallet.address().await))
            .finish_output()?,
        NftOutputBuilder::new_with_amount(1_000_000, NftId::null())
            .with_unlock_conditions([
                UnlockCondition::Address(AddressUnlockCondition::new(wallet.address().await)),
                UnlockCondition::Expiration(ExpirationUnlockCondition::new(
                    wallet.address().await,
                    wallet.client().get_slot_index().await? + 5000,
                )?),
            ])
            .add_immutable_feature(IssuerFeature::new(wallet.address().await))
            .finish_output()?,
        NftOutputBuilder::new_with_amount(1_000_000, NftId::null())
            .with_unlock_conditions([UnlockCondition::Address(AddressUnlockCondition::new(
                wallet.address().await,
            ))])
            .add_immutable_feature(Feature::Metadata(
                Irc27Metadata::new(
                    "video/mp4",
                    "https://ipfs.io/ipfs/QmPoYcVm9fx47YXNTkhpMEYSxCD3Bqh7PJYr7eo5YjLgiT"
                        .parse()
                        .unwrap(),
                    format!("Alphanet OG NFT 0"),
                )
                .with_issuer_name("Rekt wallet")
                .with_collection_name("Alpha")
                .try_into()?,
            ))
            .finish_output()?,
        DelegationOutputBuilder::new_with_amount(1_000_000, DelegationId::null(), account_address)
            .add_unlock_condition(AddressUnlockCondition::new(wallet.address().await))
            .with_start_epoch(protocol_parameters.delegation_start_epoch(latest_slot_commitment_id))
            .finish_output()?,
        AccountOutputBuilder::from(account_output_data.output.as_account())
            .with_account_id(*account_address.account_id())
            .with_mana(account_output_data.output.available_mana(
                &protocol_parameters,
                account_output_data.output_id.transaction_id().slot_index(),
                wallet.client().get_slot_index().await?,
            )?)
            .with_foundry_counter(account_output_data.output.as_account().foundry_counter() + 1)
            .finish_output()?,
        FoundryOutputBuilder::new_with_minimum_amount(
            storage_score_params,
            account_output_data.output.as_account().foundry_counter() + 1,
            TokenScheme::Simple(SimpleTokenScheme::new(100, 0, 1000)?),
        )
        .add_unlock_condition(ImmutableAccountAddressUnlockCondition::new(account_address))
        .add_immutable_feature(Feature::Metadata(
            Irc30Metadata::new("My Native Token", "REKT", 10)
                .with_description("A native token to test the iota-sdk.")
                .try_into()?,
        ))
        .finish_output()?,
    ];

    let transaction = wallet.send_outputs(outputs, None).await?;
    println!(
        "Transaction {} sent\nBlock {:?}\n{transaction:?}",
        transaction.transaction_id, transaction.block_id
    );

    Ok(())
}
