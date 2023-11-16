// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// Address types used in the wallet
pub(crate) mod address;
pub(crate) mod balance;
#[cfg(feature = "participation")]
pub mod participation;

use std::str::FromStr;

use crypto::keys::bip44::Bip44;
use serde::{Deserialize, Serialize};

pub use self::{
    address::{AddressWithUnspentOutputs, Bip44Address},
    balance::{Balance, BaseCoinBalance, NativeTokensBalance, RequiredStorageDeposit},
};
use crate::{
    client::secret::types::InputSigningData,
    types::{
        api::core::OutputWithMetadataResponse,
        block::{
            address::Address,
            output::{dto::OutputDto, Output, OutputId, OutputMetadata},
            payload::signed_transaction::{dto::SignedTransactionPayloadDto, SignedTransactionPayload, TransactionId},
            slot::SlotIndex,
            BlockId, Error as BlockError,
        },
        TryFromDto,
    },
    utils::serde::bip44::option_bip44,
    wallet::core::WalletData,
};

/// An output with metadata
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputData {
    /// The output id
    pub output_id: OutputId,
    pub metadata: OutputMetadata,
    /// The actual Output
    pub output: Output,
    /// If an output is spent
    pub is_spent: bool,
    /// Associated wallet address.
    pub address: Address,
    /// Network ID
    pub network_id: u64,
    pub remainder: bool,
    // bip44 path
    pub chain: Option<Bip44>,
}

impl OutputData {
    pub fn input_signing_data(
        &self,
        wallet_data: &WalletData,
        slot_index: SlotIndex,
    ) -> crate::wallet::Result<Option<InputSigningData>> {
        let (unlock_address, _unlocked_account_or_nft_address) =
            self.output.required_and_unlocked_address(slot_index, &self.output_id)?;

        let chain = if unlock_address == self.address {
            self.chain
        } else if let Address::Ed25519(_) = unlock_address {
            if wallet_data.address.inner() == &unlock_address {
                // TODO #1279: do we need a check to make sure that `wallet_data.address` and `wallet_data.bip_path` are
                // never conflicting?
                wallet_data.bip_path
            } else {
                return Ok(None);
            }
        } else {
            // Account and NFT addresses have no chain
            None
        };

        Ok(Some(InputSigningData {
            output: self.output.clone(),
            output_metadata: self.metadata,
            chain,
        }))
    }
}

/// Dto for an output with metadata
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputDataDto {
    /// The output id
    pub output_id: OutputId,
    /// The metadata of the output
    pub metadata: OutputMetadata,
    /// The actual Output
    pub output: OutputDto,
    /// If an output is spent
    pub is_spent: bool,
    /// Associated account address.
    pub address: Address,
    /// Network ID
    pub network_id: String,
    /// Remainder
    pub remainder: bool,
    /// Bip32 path
    #[serde(with = "option_bip44", default)]
    pub chain: Option<Bip44>,
}

impl From<&OutputData> for OutputDataDto {
    fn from(value: &OutputData) -> Self {
        Self {
            output_id: value.output_id,
            metadata: value.metadata,
            output: OutputDto::from(&value.output),
            is_spent: value.is_spent,
            address: value.address.clone(),
            network_id: value.network_id.to_string(),
            remainder: value.remainder,
            chain: value.chain,
        }
    }
}

impl TryFromDto for OutputData {
    type Dto = OutputDataDto;
    type Error = BlockError;

    fn try_from_dto_with_params_inner(
        dto: Self::Dto,
        params: crate::types::ValidationParams<'_>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            output_id: dto.output_id,
            metadata: dto.metadata,
            output: Output::try_from_dto_with_params(dto.output, params)?,
            is_spent: dto.is_spent,
            address: dto.address,
            network_id: dto
                .network_id
                .parse()
                .map_err(|_| BlockError::InvalidField("network id"))?,
            remainder: dto.remainder,
            chain: dto.chain,
        })
    }
}

/// A transaction with metadata
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionWithMetadata {
    pub payload: SignedTransactionPayload,
    pub block_id: Option<BlockId>,
    pub inclusion_state: InclusionState,
    // Transaction creation time
    pub timestamp: u128,
    pub transaction_id: TransactionId,
    // network id to ignore outputs when set_client_options is used to switch to another network
    pub network_id: u64,
    // set if the transaction was created by the wallet or if it was sent by someone else and is incoming
    pub incoming: bool,
    pub note: Option<String>,
    /// Outputs that are used as input in the transaction. May not be all, because some may have already been deleted
    /// from the node.
    // serde(default) is needed so it doesn't break with old dbs
    pub inputs: Vec<OutputWithMetadataResponse>,
}

/// Dto for a transaction with metadata
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionWithMetadataDto {
    /// The transaction payload
    pub payload: SignedTransactionPayloadDto,
    /// BlockId when it got sent to the Tangle
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_id: Option<BlockId>,
    /// Inclusion state of the transaction
    pub inclusion_state: InclusionState,
    /// Timestamp
    pub timestamp: String,
    pub transaction_id: TransactionId,
    /// Network id to ignore outputs when set_client_options is used to switch to another network
    pub network_id: String,
    /// If the transaction was created by the wallet or if it was sent by someone else and is incoming
    pub incoming: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    pub inputs: Vec<OutputWithMetadataResponse>,
}

impl From<&TransactionWithMetadata> for TransactionWithMetadataDto {
    fn from(value: &TransactionWithMetadata) -> Self {
        Self {
            payload: SignedTransactionPayloadDto::from(&value.payload),
            block_id: value.block_id,
            inclusion_state: value.inclusion_state,
            timestamp: value.timestamp.to_string(),
            transaction_id: value.transaction_id,
            network_id: value.network_id.to_string(),
            incoming: value.incoming,
            note: value.note.clone(),
            inputs: value.inputs.clone(),
        }
    }
}

impl TryFromDto for TransactionWithMetadata {
    type Dto = TransactionWithMetadataDto;
    type Error = BlockError;

    fn try_from_dto_with_params_inner(
        dto: Self::Dto,
        params: crate::types::ValidationParams<'_>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            payload: SignedTransactionPayload::try_from_dto_with_params(dto.payload, params)?,
            block_id: dto.block_id,
            inclusion_state: dto.inclusion_state,
            timestamp: dto
                .timestamp
                .parse()
                .map_err(|_| BlockError::InvalidField("timestamp"))?,
            transaction_id: dto.transaction_id,
            network_id: dto
                .network_id
                .parse()
                .map_err(|_| BlockError::InvalidField("network id"))?,
            incoming: dto.incoming,
            note: dto.note,
            inputs: dto.inputs,
        })
    }
}

/// Possible InclusionStates for transactions
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum InclusionState {
    Pending,
    Confirmed,
    Conflicting,
    UnknownPruned,
}

/// The output kind enum.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum OutputKind {
    /// Account output.
    Account,
    /// Basic output.
    Basic,
    /// Foundry output.
    Foundry,
    /// Nft output.
    Nft,
}

impl FromStr for OutputKind {
    type Err = crate::wallet::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let kind = match s {
            "Account" => Self::Account,
            "Basic" => Self::Basic,
            "Foundry" => Self::Foundry,
            "Nft" => Self::Nft,
            _ => return Err(crate::wallet::Error::InvalidOutputKind(s.to_string())),
        };
        Ok(kind)
    }
}