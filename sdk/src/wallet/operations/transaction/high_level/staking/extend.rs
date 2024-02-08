// Copyright 2024 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    client::{api::PreparedTransactionData, secret::SecretManage},
    types::block::{
        context_input::{ContextInput, RewardContextInput},
        output::{feature::StakingFeature, AccountId, AccountOutputBuilder},
    },
    wallet::{types::TransactionWithMetadata, TransactionOptions, Wallet},
};

impl<S: 'static + SecretManage> Wallet<S>
where
    crate::wallet::Error: From<S::Error>,
    crate::client::Error: From<S::Error>,
{
    pub async fn extend_staking(
        &self,
        account_id: AccountId,
        additional_epochs: u32,
    ) -> crate::wallet::Result<TransactionWithMetadata> {
        let prepared = self.prepare_extend_staking(account_id, additional_epochs).await?;

        self.sign_and_submit_transaction(prepared, None, None).await
    }

    /// Prepares the transaction for [Wallet::extend_staking()].
    pub async fn prepare_extend_staking(
        &self,
        account_id: AccountId,
        additional_epochs: u32,
    ) -> crate::wallet::Result<PreparedTransactionData> {
        log::debug!("[TRANSACTION] prepare_extend_staking");

        let account_output_data = self
            .data()
            .await
            .unspent_account_output(&account_id)
            .cloned()
            .ok_or_else(|| crate::wallet::Error::AccountNotFound)?;

        let protocol_parameters = self.client().get_protocol_parameters().await?;

        let slot_commitment_id = self.client().get_issuance().await?.latest_commitment.id();

        let future_bounded_epoch =
            protocol_parameters.epoch_index_of(protocol_parameters.future_bounded_slot(slot_commitment_id));

        let staking_feature = account_output_data
            .output
            .features()
            .and_then(|f| f.staking())
            .ok_or_else(|| crate::wallet::Error::StakingFailed(format!("account id {account_id} is not staking")))?;

        let mut output_builder =
            AccountOutputBuilder::from(account_output_data.output.as_account()).with_account_id(account_id);

        let mut options = TransactionOptions::default();

        // Just extend the end epoch if it's still possible
        if future_bounded_epoch <= staking_feature.end_epoch() {
            output_builder = output_builder.replace_feature(StakingFeature::new(
                staking_feature.staked_amount(),
                staking_feature.fixed_cost(),
                staking_feature.start_epoch(),
                staking_feature.end_epoch() + additional_epochs,
            ));
        // Otherwise, we'll have to claim the rewards
        } else {
            if additional_epochs < protocol_parameters.staking_unbonding_period() {
                return Err(crate::wallet::Error::StakingFailed(format!(
                    "new staking period {additional_epochs} is less than the minimum {}",
                    protocol_parameters.staking_unbonding_period()
                )));
            }
            let past_bounded_epoch =
                protocol_parameters.epoch_index_of(protocol_parameters.past_bounded_slot(slot_commitment_id));
            let end_epoch = past_bounded_epoch + additional_epochs;
            output_builder = output_builder.replace_feature(StakingFeature::new(
                staking_feature.staked_amount(),
                staking_feature.fixed_cost(),
                past_bounded_epoch,
                end_epoch,
            ));
            options.custom_inputs = Some(vec![account_output_data.output_id]);
            options.context_inputs = Some(vec![ContextInput::from(RewardContextInput::new(0)?)]);
        }

        let output = output_builder.finish_output()?;

        let transaction = self.prepare_transaction([output], options).await?;

        Ok(transaction)
    }
}