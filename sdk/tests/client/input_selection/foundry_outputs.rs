// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use iota_sdk::{
    client::{
        api::input_selection::{Burn, Error, InputSelection, Requirement},
        secret::types::InputSigningData,
    },
    types::block::{
        address::{AccountAddress, Address},
        output::{
            unlock_condition::AddressUnlockCondition, AccountId, AccountOutputBuilder, FoundryId, Output,
            SimpleTokenScheme, TokenId,
        },
        protocol::protocol_parameters,
        rand::output::{rand_output_id_with_slot_index, rand_output_metadata_with_id},
    },
};
use pretty_assertions::assert_eq;

use crate::client::{
    build_inputs, build_outputs, is_remainder_or_return, unsorted_eq,
    Build::{Account, Basic, Foundry},
    ACCOUNT_ID_1, ACCOUNT_ID_2, BECH32_ADDRESS_ED25519_0, SLOT_COMMITMENT_ID, SLOT_INDEX,
};

#[test]
fn missing_input_account_for_foundry() {
    let protocol_parameters = protocol_parameters();
    let account_id_2 = AccountId::from_str(ACCOUNT_ID_2).unwrap();

    let inputs = build_inputs(
        [(
            Basic {
                amount: 1_000_000,
                address: Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
                native_token: None,
                sender: None,
                sdruc: None,
                timelock: None,
                expiration: None,
            },
            None,
        )],
        Some(SLOT_INDEX),
    );
    let outputs = build_outputs([Foundry {
        amount: 1_000_000,
        account_id: account_id_2,
        serial_number: 1,
        token_scheme: SimpleTokenScheme::new(0, 0, 10).unwrap(),
        native_token: None,
    }]);

    let selected = InputSelection::new(
        inputs,
        None,
        outputs,
        [Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap()],
        SLOT_INDEX,
        SLOT_COMMITMENT_ID,
        protocol_parameters,
    )
    .select();

    assert!(matches!(
        selected,
        Err(Error::UnfulfillableRequirement(Requirement::Account(account_id))) if account_id == account_id_2
    ));
}

// #[test]
// fn existing_input_account_for_foundry_account() {
//     let protocol_parameters = protocol_parameters();
//     let account_id_2 = AccountId::from_str(ACCOUNT_ID_2).unwrap();

//     let inputs = build_inputs([Account(
//         1_255_500,
//         account_id_2,
//         0,
//         Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
//         Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
//         None,
//         None,
//         None,
//         None,
//     )]);
//     let outputs = build_outputs([Foundry(
//         1_000_000,
//         account_id_2,
//         1,
//         SimpleTokenScheme::new(0, 0, 10).unwrap(),
//         None,
//     )]);

//     let selected = InputSelection::new(
//         inputs.clone(),
//         outputs,
//         [Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap()],
//         protocol_parameters,
//     )
//     .select()
//     .unwrap();

//     assert!(unsorted_eq(&selected.inputs, &inputs));
//     // Account next state + foundry
//     assert_eq!(selected.outputs.len(), 2);
// }

#[test]
fn minted_native_tokens_in_new_remainder() {
    let protocol_parameters = protocol_parameters();
    let account_id_2 = AccountId::from_str(ACCOUNT_ID_2).unwrap();

    let inputs = build_inputs(
        [
            (
                Basic {
                    amount: 1_000_000,
                    address: Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
                    native_token: None,
                    sender: None,
                    sdruc: None,
                    timelock: None,
                    expiration: None,
                },
                None,
            ),
            (
                Account {
                    amount: 1_000_000,
                    account_id: account_id_2,
                    address: Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
                    sender: None,
                    issuer: None,
                },
                None,
            ),
        ],
        Some(SLOT_INDEX),
    );
    let outputs = build_outputs([Foundry {
        amount: 1_000_000,
        account_id: account_id_2,
        serial_number: 1,
        token_scheme: SimpleTokenScheme::new(10, 0, 10).unwrap(),
        native_token: None,
    }]);

    let selected = InputSelection::new(
        inputs.clone(),
        None,
        outputs,
        [Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap()],
        SLOT_INDEX,
        SLOT_COMMITMENT_ID,
        protocol_parameters,
    )
    .select()
    .unwrap();

    assert!(unsorted_eq(&selected.inputs, &inputs));
    // Account next state + foundry + basic output with native tokens
    assert_eq!(selected.outputs.len(), 3);
    selected.outputs.iter().for_each(|output| {
        if let Output::Basic(_basic_output) = &output {
            // Basic output remainder has the minted native tokens
            // TODO reenable when ISA supports NTs again
            // assert_eq!(basic_output.native_token().unwrap().amount().as_u32(), 10);
        }
    });
}

#[test]
fn minted_native_tokens_in_provided_output() {
    let protocol_parameters = protocol_parameters();
    let account_id_2 = AccountId::from_str(ACCOUNT_ID_2).unwrap();
    let foundry_id = FoundryId::build(&AccountAddress::from(account_id_2), 1, SimpleTokenScheme::KIND);
    let token_id = TokenId::from(foundry_id);

    let inputs = build_inputs(
        [
            (
                Basic {
                    amount: 2_000_000,
                    address: Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
                    native_token: None,
                    sender: None,
                    sdruc: None,
                    timelock: None,
                    expiration: None,
                },
                None,
            ),
            (
                Account {
                    amount: 1_000_000,
                    account_id: account_id_2,
                    address: Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
                    sender: None,
                    issuer: None,
                },
                None,
            ),
        ],
        Some(SLOT_INDEX),
    );
    let outputs = build_outputs([
        Foundry {
            amount: 1_000_000,
            account_id: account_id_2,
            serial_number: 1,
            token_scheme: SimpleTokenScheme::new(100, 0, 100).unwrap(),
            native_token: None,
        },
        Basic {
            amount: 1_000_000,
            address: Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
            native_token: Some((&token_id.to_string(), 100)),
            sender: None,
            sdruc: None,
            timelock: None,
            expiration: None,
        },
    ]);

    let selected = InputSelection::new(
        inputs.clone(),
        None,
        outputs.clone(),
        [Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap()],
        SLOT_INDEX,
        SLOT_COMMITMENT_ID,
        protocol_parameters,
    )
    .select()
    .unwrap();

    assert!(unsorted_eq(&selected.inputs, &inputs));
    assert_eq!(selected.outputs.len(), 3);
    assert!(selected.outputs.contains(&outputs[0]));
    assert!(selected.outputs.contains(&outputs[1]));
    assert!(selected.outputs.iter().any(|output| output.is_account()));
}

#[test]
fn melt_native_tokens() {
    let protocol_parameters = protocol_parameters();
    let account_id_1 = AccountId::from_str(ACCOUNT_ID_1).unwrap();

    let mut inputs = build_inputs(
        [
            (
                Basic {
                    amount: 1_000_000,
                    address: Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
                    native_token: None,
                    sender: None,
                    sdruc: None,
                    timelock: None,
                    expiration: None,
                },
                None,
            ),
            (
                Foundry {
                    amount: 1_000_000,
                    account_id: account_id_1,
                    serial_number: 1,
                    token_scheme: SimpleTokenScheme::new(10, 0, 10).unwrap(),
                    native_token: Some((
                        "0x0811111111111111111111111111111111111111111111111111111111111111110100000000",
                        10,
                    )),
                },
                None,
            ),
        ],
        Some(SLOT_INDEX),
    );
    let account_output = AccountOutputBuilder::new_with_amount(1_000_000, account_id_1)
        .add_unlock_condition(AddressUnlockCondition::new(
            Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
        ))
        .with_foundry_counter(1)
        .finish_output()
        .unwrap();
    inputs.push(InputSigningData {
        output: account_output,
        output_metadata: rand_output_metadata_with_id(rand_output_id_with_slot_index(SLOT_INDEX)),
        chain: None,
    });
    let outputs = build_outputs([Foundry {
        amount: 1_000_000,
        account_id: account_id_1,
        serial_number: 1,
        token_scheme: SimpleTokenScheme::new(10, 5, 10).unwrap(),
        native_token: None,
    }]);

    let selected = InputSelection::new(
        inputs.clone(),
        None,
        outputs,
        [Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap()],
        SLOT_INDEX,
        SLOT_COMMITMENT_ID,
        protocol_parameters,
    )
    .select()
    .unwrap();

    assert!(unsorted_eq(&selected.inputs, &inputs));
    // Account next state + foundry + basic output with native tokens
    assert_eq!(selected.outputs.len(), 3);
    selected.outputs.iter().for_each(|output| {
        if let Output::Basic(_basic_output) = &output {
            // Basic output remainder has the remaining native tokens
            // TODO reenable when ISA supports NTs again
            // assert_eq!(basic_output.native_token().unwrap().amount().as_u32(), 5);
        }
    });
}

#[test]
fn destroy_foundry_with_account_state_transition() {
    let protocol_parameters = protocol_parameters();
    let account_id_2 = AccountId::from_str(ACCOUNT_ID_2).unwrap();

    let inputs = build_inputs(
        [
            (
                Account {
                    amount: 50_300,
                    account_id: account_id_2,
                    address: Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
                    sender: None,
                    issuer: None,
                },
                None,
            ),
            (
                Foundry {
                    amount: 52_800,
                    account_id: account_id_2,
                    serial_number: 1,
                    token_scheme: SimpleTokenScheme::new(10, 10, 10).unwrap(),
                    native_token: None,
                },
                None,
            ),
        ],
        Some(SLOT_INDEX),
    );
    let account_output = AccountOutputBuilder::from(inputs[0].output.as_account())
        .with_amount(103_100)
        .finish_output()
        .unwrap();
    // Account output gets the amount from the foundry output added
    let outputs = [account_output];

    let selected = InputSelection::new(
        inputs.clone(),
        None,
        outputs,
        [Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap()],
        SLOT_INDEX,
        SLOT_COMMITMENT_ID,
        protocol_parameters,
    )
    .with_burn(Burn::new().add_foundry(inputs[1].output.as_foundry().id()))
    .select()
    .unwrap();

    assert!(unsorted_eq(&selected.inputs, &inputs));
    // Account next state
    assert_eq!(selected.outputs.len(), 1);
}

#[test]
fn destroy_foundry_with_account_burn() {
    let protocol_parameters = protocol_parameters();
    let account_id_2 = AccountId::from_str(ACCOUNT_ID_2).unwrap();

    let inputs = build_inputs(
        [
            (
                Account {
                    amount: 1_000_000,
                    account_id: account_id_2,
                    address: Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
                    sender: None,
                    issuer: None,
                },
                None,
            ),
            (
                Foundry {
                    amount: 1_000_000,
                    account_id: account_id_2,
                    serial_number: 1,
                    token_scheme: SimpleTokenScheme::new(10, 10, 10).unwrap(),
                    native_token: None,
                },
                None,
            ),
        ],
        Some(SLOT_INDEX),
    );
    let outputs = build_outputs([Basic {
        amount: 1_000_000,
        address: Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
        native_token: None,
        sender: None,
        sdruc: None,
        timelock: None,
        expiration: None,
    }]);

    let selected = InputSelection::new(
        inputs.clone(),
        None,
        outputs.clone(),
        [Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap()],
        SLOT_INDEX,
        SLOT_COMMITMENT_ID,
        protocol_parameters,
    )
    .with_burn(
        Burn::new()
            .add_foundry(inputs[1].output.as_foundry().id())
            .add_account(account_id_2),
    )
    .select()
    .unwrap();

    assert!(unsorted_eq(&selected.inputs, &inputs));
    assert_eq!(selected.outputs.len(), 2);
    assert!(selected.outputs.contains(&outputs[0]));
    selected.outputs.iter().for_each(|output| {
        if !outputs.contains(output) {
            assert!(is_remainder_or_return(
                output,
                1_000_000,
                Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
                None,
            ));
        }
    });
}

#[test]
fn prefer_basic_to_foundry() {
    let protocol_parameters = protocol_parameters();
    let account_id_1 = AccountId::from_str(ACCOUNT_ID_1).unwrap();

    let inputs = build_inputs(
        [
            (
                Account {
                    amount: 1_000_000,
                    account_id: account_id_1,
                    address: Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
                    sender: None,
                    issuer: None,
                },
                None,
            ),
            (
                Foundry {
                    amount: 1_000_000,
                    account_id: account_id_1,
                    serial_number: 1,
                    token_scheme: SimpleTokenScheme::new(10, 10, 10).unwrap(),
                    native_token: None,
                },
                None,
            ),
            (
                Basic {
                    amount: 1_000_000,
                    address: Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
                    native_token: None,
                    sender: None,
                    sdruc: None,
                    timelock: None,
                    expiration: None,
                },
                None,
            ),
        ],
        Some(SLOT_INDEX),
    );
    let outputs = build_outputs([Basic {
        amount: 1_000_000,
        address: Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
        native_token: None,
        sender: None,
        sdruc: None,
        timelock: None,
        expiration: None,
    }]);

    let selected = InputSelection::new(
        inputs.clone(),
        None,
        outputs.clone(),
        [Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap()],
        SLOT_INDEX,
        SLOT_COMMITMENT_ID,
        protocol_parameters,
    )
    .select()
    .unwrap();

    assert_eq!(selected.inputs.len(), 1);
    assert_eq!(selected.inputs[0], inputs[2]);
    assert_eq!(selected.outputs, outputs);
}

#[test]
fn simple_foundry_transition_basic_not_needed() {
    let protocol_parameters = protocol_parameters();
    let account_id_1 = AccountId::from_str(ACCOUNT_ID_1).unwrap();

    let mut inputs = build_inputs(
        [
            (
                Basic {
                    amount: 1_000_000,
                    address: Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
                    native_token: None,
                    sender: None,
                    sdruc: None,
                    timelock: None,
                    expiration: None,
                },
                None,
            ),
            (
                Foundry {
                    amount: 1_000_000,
                    account_id: account_id_1,
                    serial_number: 1,
                    token_scheme: SimpleTokenScheme::new(10, 10, 10).unwrap(),
                    native_token: None,
                },
                None,
            ),
        ],
        Some(SLOT_INDEX),
    );
    let account_output = AccountOutputBuilder::new_with_amount(2_000_000, account_id_1)
        .add_unlock_condition(AddressUnlockCondition::new(
            Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
        ))
        .with_foundry_counter(1)
        .finish_output()
        .unwrap();
    inputs.push(InputSigningData {
        output: account_output,
        output_metadata: rand_output_metadata_with_id(rand_output_id_with_slot_index(SLOT_INDEX)),
        chain: None,
    });

    let outputs = build_outputs([Foundry {
        amount: 1_000_000,
        account_id: account_id_1,
        serial_number: 1,
        token_scheme: SimpleTokenScheme::new(10, 10, 10).unwrap(),
        native_token: None,
    }]);

    let selected = InputSelection::new(
        inputs.clone(),
        None,
        outputs.clone(),
        [Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap()],
        SLOT_INDEX,
        SLOT_COMMITMENT_ID,
        protocol_parameters,
    )
    .select()
    .unwrap();

    assert_eq!(selected.inputs.len(), 2);
    assert!(selected.inputs.contains(&inputs[1]));
    assert!(selected.inputs.contains(&inputs[2]));
    assert_eq!(selected.outputs.len(), 2);
    assert!(selected.outputs.contains(&outputs[0]));
    selected.outputs.iter().for_each(|output| {
        if !outputs.contains(output) {
            assert!(output.is_account());
            assert_eq!(output.amount(), 2_000_000);
            assert_eq!(*output.as_account().account_id(), account_id_1);
            assert_eq!(output.as_account().unlock_conditions().len(), 1);
            assert_eq!(output.as_account().features().len(), 0);
            assert_eq!(output.as_account().immutable_features().len(), 0);
            assert_eq!(
                *output.as_account().address(),
                Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap()
            );
        }
    });
}

#[test]
fn simple_foundry_transition_basic_not_needed_with_remainder() {
    let protocol_parameters = protocol_parameters();
    let account_id_1 = AccountId::from_str(ACCOUNT_ID_1).unwrap();

    let mut inputs = build_inputs(
        [
            (
                Basic {
                    amount: 1_000_000,
                    address: Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
                    native_token: None,
                    sender: None,
                    sdruc: None,
                    timelock: None,
                    expiration: None,
                },
                None,
            ),
            (
                Foundry {
                    amount: 2_000_000,
                    account_id: account_id_1,
                    serial_number: 1,
                    token_scheme: SimpleTokenScheme::new(10, 10, 10).unwrap(),
                    native_token: None,
                },
                None,
            ),
        ],
        Some(SLOT_INDEX),
    );
    let account_output = AccountOutputBuilder::new_with_amount(2_000_000, account_id_1)
        .add_unlock_condition(AddressUnlockCondition::new(
            Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
        ))
        .with_foundry_counter(1)
        .finish_output()
        .unwrap();
    inputs.push(InputSigningData {
        output: account_output,
        output_metadata: rand_output_metadata_with_id(rand_output_id_with_slot_index(SLOT_INDEX)),
        chain: None,
    });
    let outputs = build_outputs([Foundry {
        amount: 1_000_000,
        account_id: account_id_1,
        serial_number: 1,
        token_scheme: SimpleTokenScheme::new(10, 10, 10).unwrap(),
        native_token: None,
    }]);

    let selected = InputSelection::new(
        inputs.clone(),
        None,
        outputs.clone(),
        [Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap()],
        SLOT_INDEX,
        SLOT_COMMITMENT_ID,
        protocol_parameters,
    )
    .select()
    .unwrap();

    assert_eq!(selected.inputs.len(), 2);
    assert!(selected.inputs.contains(&inputs[1]));
    assert!(selected.inputs.contains(&inputs[2]));
    assert_eq!(selected.outputs.len(), 3);
    assert!(selected.outputs.contains(&outputs[0]));
    selected.outputs.iter().for_each(|output| {
        if !outputs.contains(output) {
            if output.is_account() {
                assert_eq!(output.amount(), 2_000_000);
                assert_eq!(*output.as_account().account_id(), account_id_1);
                assert_eq!(output.as_account().unlock_conditions().len(), 1);
                assert_eq!(output.as_account().features().len(), 0);
                assert_eq!(output.as_account().immutable_features().len(), 0);
                assert_eq!(
                    *output.as_account().address(),
                    Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap()
                );
            } else if output.is_basic() {
                assert!(is_remainder_or_return(
                    output,
                    1_000_000,
                    Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
                    None,
                ));
            } else {
                panic!("unexpected output type")
            }
        }
    });
}

// TODO
// #[test]
// fn account_required_through_sender_and_sufficient() {
//     let protocol_parameters = protocol_parameters();
//     let account_id_1 = AccountId::from_str(ACCOUNT_ID_1).unwrap();

//     let mut inputs = build_inputs([(BECH32_ADDRESS, 1_000_000, None)]);
//     inputs.extend(build_input_signing_data_foundry_outputs([(
//         account_id_1,
//         2_000_000,
//         SimpleTokenScheme::new(10, 10, 10).unwrap(),
//         None,
//     )]));
//     inputs.extend(build_inputs([(
//         account_id_1,
//         BECH32_ADDRESS,
//         2_000_000,
//         None,
//     )]));
//     let outputs = build_outputs([Basic(
//         1_000_000,
//         BECH32_ADDRESS,
//         None,
//         Some(BECH32_ADDRESS_ACCOUNT_SENDER),
//     )];

//     let selected = InputSelection::new(inputs.clone(), outputs.clone(), [],protocol_parameters)
//         .select()
//         .unwrap();

//     assert_eq!(selected.inputs.len(), 1);
//     assert!(selected.inputs.contains(&inputs[2]));
//     // assert_eq!(selected.outputs.len(), 3);
//     // assert!(selected.outputs.contains(&outputs[0]));
//     // selected.outputs.iter().for_each(|output| {
//     //     if !outputs.contains(output) {
//     //         if output.is_account() {
//     //             assert_eq!(output.amount(), 2_000_000);
//     //             assert_eq!(output.as_account().native_tokens().len(), 0);
//     //             assert_eq!(*output.as_account().account_id(), account_id_1);
//     //             assert_eq!(output.as_account().unlock_conditions().len(), 1);
//     //             assert_eq!(output.as_account().features().len(), 0);
//     //             assert_eq!(output.as_account().immutable_features().len(), 0);
//     //             assert_eq!(
//     //                 *output.as_account().state_controller_address(),
//     //                 Address::try_from_bech32(BECH32_ADDRESS).unwrap().1
//     //             );
//     //             assert_eq!(
//     //                 *output.as_account().governor_address(),
//     //                 Address::try_from_bech32(BECH32_ADDRESS).unwrap().1
//     //             );
//     //         } else if output.is_basic() {
//     //             assert_eq!(output.amount(), 1_000_000);
//     //             assert_eq!(output.as_basic().native_tokens().len(), 0);
//     //             assert_eq!(output.as_basic().unlock_conditions().len(), 1);
//     //             assert_eq!(output.as_basic().features().len(), 0);
//     //             assert_eq!(
//     //                 *output.as_basic().address(),
//     //                 Address::try_from_bech32(BECH32_ADDRESS).unwrap().1
//     //             );
//     //         } else {
//     //             panic!("unexpected output type")
//     //         }
//     //     }
//     // });
// }

#[test]
fn mint_and_burn_at_the_same_time() {
    let protocol_parameters = protocol_parameters();
    let account_id_1 = AccountId::from_str(ACCOUNT_ID_1).unwrap();
    let foundry_id = FoundryId::build(&AccountAddress::from(account_id_1), 1, SimpleTokenScheme::KIND);
    let token_id = TokenId::from(foundry_id);

    let mut inputs = build_inputs(
        [(
            Foundry {
                amount: 1_000_000,
                account_id: account_id_1,
                serial_number: 1,
                token_scheme: SimpleTokenScheme::new(100, 0, 200).unwrap(),
                native_token: Some((&token_id.to_string(), 100)),
            },
            None,
        )],
        Some(SLOT_INDEX),
    );
    let account_output = AccountOutputBuilder::new_with_amount(2_000_000, account_id_1)
        .add_unlock_condition(AddressUnlockCondition::new(
            Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
        ))
        .with_foundry_counter(1)
        .finish_output()
        .unwrap();
    inputs.push(InputSigningData {
        output: account_output,
        output_metadata: rand_output_metadata_with_id(rand_output_id_with_slot_index(SLOT_INDEX)),
        chain: None,
    });

    let outputs = build_outputs([Foundry {
        amount: 1_000_000,
        account_id: account_id_1,
        serial_number: 1,
        token_scheme: SimpleTokenScheme::new(120, 0, 200).unwrap(),
        native_token: Some((&token_id.to_string(), 110)),
    }]);

    let selected = InputSelection::new(
        inputs.clone(),
        None,
        outputs,
        [Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap()],
        SLOT_INDEX,
        SLOT_COMMITMENT_ID,
        protocol_parameters,
    )
    .with_burn(Burn::new().add_native_token(token_id, 10))
    .select();

    assert!(matches!(
        selected,
        Err(Error::UnfulfillableRequirement(Requirement::Foundry(id))) if id == foundry_id
    ));
}

#[test]
fn take_amount_from_account_and_foundry_to_fund_basic() {
    let protocol_parameters = protocol_parameters();
    let account_id_1 = AccountId::from_str(ACCOUNT_ID_1).unwrap();
    let foundry_id = FoundryId::build(&AccountAddress::from(account_id_1), 0, SimpleTokenScheme::KIND);
    let token_id = TokenId::from(foundry_id);

    let mut inputs = build_inputs(
        [
            (
                Basic {
                    amount: 1_000_000,
                    address: Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
                    native_token: None,
                    sender: None,
                    sdruc: None,
                    timelock: None,
                    expiration: None,
                },
                None,
            ),
            (
                Foundry {
                    amount: 1_000_000,
                    account_id: account_id_1,
                    serial_number: 1,
                    token_scheme: SimpleTokenScheme::new(100, 0, 200).unwrap(),
                    native_token: Some((&token_id.to_string(), 100)),
                },
                None,
            ),
        ],
        Some(SLOT_INDEX),
    );
    let account_output = AccountOutputBuilder::new_with_amount(2_000_000, account_id_1)
        .add_unlock_condition(AddressUnlockCondition::new(
            Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
        ))
        .with_foundry_counter(1)
        .finish_output()
        .unwrap();
    inputs.push(InputSigningData {
        output: account_output,
        output_metadata: rand_output_metadata_with_id(rand_output_id_with_slot_index(SLOT_INDEX)),
        chain: None,
    });
    let outputs = build_outputs([Basic {
        amount: 3_200_000,
        address: Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
        native_token: None,
        sender: None,
        sdruc: None,
        timelock: None,
        expiration: None,
    }]);

    let selected = InputSelection::new(
        inputs.clone(),
        None,
        outputs.clone(),
        [Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap()],
        SLOT_INDEX,
        SLOT_COMMITMENT_ID,
        protocol_parameters,
    )
    .select()
    .unwrap();

    assert!(unsorted_eq(&selected.inputs, &inputs));
    assert_eq!(selected.outputs.len(), 3);
    assert!(selected.outputs.contains(&outputs[0]));
    assert!(selected.outputs.iter().any(|output| output.is_account()));
    assert!(selected.outputs.iter().any(|output| output.is_foundry()));
    assert_eq!(
        selected.outputs.iter().map(|output| output.amount()).sum::<u64>(),
        4_000_000
    );
}

#[test]
fn create_native_token_but_burn_account() {
    let protocol_parameters = protocol_parameters();
    let account_id_1 = AccountId::from_str(ACCOUNT_ID_1).unwrap();
    let foundry_id = FoundryId::build(&AccountAddress::from(account_id_1), 1, SimpleTokenScheme::KIND);
    let token_id = TokenId::from(foundry_id);

    let inputs = build_inputs(
        [
            (
                Account {
                    amount: 2_000_000,
                    account_id: account_id_1,
                    address: Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
                    sender: None,
                    issuer: None,
                },
                None,
            ),
            (
                Foundry {
                    amount: 1_000_000,
                    account_id: account_id_1,
                    serial_number: 1,
                    token_scheme: SimpleTokenScheme::new(0, 0, 100).unwrap(),
                    native_token: None,
                },
                None,
            ),
        ],
        Some(SLOT_INDEX),
    );
    let outputs = build_outputs([Foundry {
        amount: 1_000_000,
        account_id: account_id_1,
        serial_number: 1,
        token_scheme: SimpleTokenScheme::new(100, 0, 100).unwrap(),
        native_token: Some((&token_id.to_string(), 100)),
    }]);

    let selected = InputSelection::new(
        inputs.clone(),
        None,
        outputs.clone(),
        [Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap()],
        SLOT_INDEX,
        SLOT_COMMITMENT_ID,
        protocol_parameters,
    )
    .with_burn(Burn::new().add_account(account_id_1))
    .select()
    .unwrap();

    assert!(unsorted_eq(&selected.inputs, &inputs));
    // One output should be added for the remainder.
    assert_eq!(selected.outputs.len(), 2);
    assert!(selected.outputs.contains(&outputs[0]));
    selected.outputs.iter().for_each(|output| {
        if !outputs.contains(output) {
            assert!(is_remainder_or_return(
                output,
                2_000_000,
                Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
                None,
            ));
        }
    });
}

#[test]
fn melted_tokens_not_provided() {
    let protocol_parameters = protocol_parameters();
    let account_id_1 = AccountId::from_str(ACCOUNT_ID_1).unwrap();
    let foundry_id = FoundryId::build(&AccountAddress::from(account_id_1), 1, SimpleTokenScheme::KIND);
    let token_id_1 = TokenId::from(foundry_id);

    let inputs = build_inputs(
        [
            (
                Account {
                    amount: 2_000_000,
                    account_id: account_id_1,
                    address: Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
                    sender: None,
                    issuer: None,
                },
                None,
            ),
            (
                Foundry {
                    amount: 1_000_000,
                    account_id: account_id_1,
                    serial_number: 1,
                    token_scheme: SimpleTokenScheme::new(100, 0, 100).unwrap(),
                    native_token: None,
                },
                None,
            ),
        ],
        Some(SLOT_INDEX),
    );
    let outputs = build_outputs([Foundry {
        amount: 1_000_000,
        account_id: account_id_1,
        serial_number: 1,
        token_scheme: SimpleTokenScheme::new(100, 100, 100).unwrap(),
        native_token: None,
    }]);

    let selected = InputSelection::new(
        inputs,
        None,
        outputs,
        [Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap()],
        SLOT_INDEX,
        SLOT_COMMITMENT_ID,
        protocol_parameters,
    )
    .select();

    assert!(matches!(
        selected,
        Err(Error::InsufficientNativeTokenAmount {
        token_id,
            found,
            required,
        }) if token_id == token_id_1 && found.as_u32() == 0 && required.as_u32() == 100));
}

#[test]
fn burned_tokens_not_provided() {
    let protocol_parameters = protocol_parameters();
    let account_id_1 = AccountId::from_str(ACCOUNT_ID_1).unwrap();
    let foundry_id = FoundryId::build(&AccountAddress::from(account_id_1), 0, SimpleTokenScheme::KIND);
    let token_id_1 = TokenId::from(foundry_id);

    let inputs = build_inputs(
        [
            (
                Account {
                    amount: 2_000_000,
                    account_id: account_id_1,
                    address: Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
                    sender: None,
                    issuer: None,
                },
                None,
            ),
            (
                Foundry {
                    amount: 1_000_000,
                    account_id: account_id_1,
                    serial_number: 1,
                    token_scheme: SimpleTokenScheme::new(100, 0, 100).unwrap(),
                    native_token: None,
                },
                None,
            ),
        ],
        Some(SLOT_INDEX),
    );
    let outputs = build_outputs([Foundry {
        amount: 1_000_000,
        account_id: account_id_1,
        serial_number: 1,
        token_scheme: SimpleTokenScheme::new(100, 0, 100).unwrap(),
        native_token: None,
    }]);

    let selected = InputSelection::new(
        inputs,
        None,
        outputs,
        [Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap()],
        SLOT_INDEX,
        SLOT_COMMITMENT_ID,
        protocol_parameters,
    )
    .with_burn(Burn::new().add_native_token(token_id_1, 100))
    .select();

    assert!(matches!(
        selected,
        Err(Error::InsufficientNativeTokenAmount {
        token_id,
            found,
            required,
        }) if token_id == token_id_1 && found.as_u32() == 0 && required.as_u32() == 100));
}

#[test]
fn foundry_in_outputs_and_required() {
    let protocol_parameters = protocol_parameters();
    let account_id_2 = AccountId::from_str(ACCOUNT_ID_2).unwrap();

    let mut inputs = build_inputs(
        [(
            Foundry {
                amount: 1_000_000,
                account_id: account_id_2,
                serial_number: 1,
                token_scheme: SimpleTokenScheme::new(0, 0, 10).unwrap(),
                native_token: None,
            },
            None,
        )],
        Some(SLOT_INDEX),
    );
    let account_output = AccountOutputBuilder::new_with_amount(1_251_500, account_id_2)
        .add_unlock_condition(AddressUnlockCondition::new(
            Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
        ))
        .with_foundry_counter(1)
        .finish_output()
        .unwrap();
    inputs.push(InputSigningData {
        output: account_output,
        output_metadata: rand_output_metadata_with_id(rand_output_id_with_slot_index(SLOT_INDEX)),
        chain: None,
    });
    let outputs = build_outputs([Foundry {
        amount: 1_000_000,
        account_id: account_id_2,
        serial_number: 1,
        token_scheme: SimpleTokenScheme::new(0, 0, 10).unwrap(),
        native_token: None,
    }]);

    let selected = InputSelection::new(
        inputs.clone(),
        None,
        outputs.clone(),
        [Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap()],
        SLOT_INDEX,
        SLOT_COMMITMENT_ID,
        protocol_parameters,
    )
    .with_required_inputs([*inputs[1].output_id()])
    .select()
    .unwrap();

    assert!(unsorted_eq(&selected.inputs, &inputs));
    assert_eq!(selected.outputs.len(), 2);
    assert!(selected.outputs.contains(&outputs[0]));
    selected.outputs.iter().for_each(|output| {
        if !outputs.contains(output) {
            assert_eq!(*output.as_account().account_id(), account_id_2);
        }
    });
}

#[test]
fn melt_and_burn_native_tokens() {
    let protocol_parameters = protocol_parameters();
    let account_id = AccountId::from_str(ACCOUNT_ID_1).unwrap();
    let foundry_id = FoundryId::build(&AccountAddress::from(account_id), 1, SimpleTokenScheme::KIND);
    let token_id = TokenId::from(foundry_id);

    let mut inputs = build_inputs(
        [
            (
                Basic {
                    amount: 1_000_000,
                    address: Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
                    native_token: None,
                    sender: None,
                    sdruc: None,
                    timelock: None,
                    expiration: None,
                },
                None,
            ),
            (
                Foundry {
                    amount: 1_000_000,
                    account_id,
                    serial_number: 1,
                    token_scheme: SimpleTokenScheme::new(1000, 0, 1000).unwrap(),
                    native_token: Some((&token_id.to_string(), 1000)),
                },
                None,
            ),
        ],
        Some(SLOT_INDEX),
    );
    let account_output = AccountOutputBuilder::new_with_amount(1_000_000, account_id)
        .add_unlock_condition(AddressUnlockCondition::new(
            Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap(),
        ))
        .with_foundry_counter(1)
        .finish_output()
        .unwrap();
    inputs.push(InputSigningData {
        output: account_output,
        output_metadata: rand_output_metadata_with_id(rand_output_id_with_slot_index(SLOT_INDEX)),
        chain: None,
    });
    let outputs = build_outputs([Foundry {
        amount: 1_000_000,
        account_id,
        serial_number: 1,
        token_scheme: SimpleTokenScheme::new(1000, 123, 1000).unwrap(),
        native_token: None,
    }]);

    let selected = InputSelection::new(
        inputs.clone(),
        None,
        outputs,
        [Address::try_from_bech32(BECH32_ADDRESS_ED25519_0).unwrap()],
        SLOT_INDEX,
        SLOT_COMMITMENT_ID,
        protocol_parameters,
    )
    // Burn 456 native tokens
    .with_burn(Burn::new().add_native_token(token_id, 456))
    .select()
    .unwrap();

    assert!(unsorted_eq(&selected.inputs, &inputs));
    // Account next state + foundry + basic output with native tokens
    assert_eq!(selected.outputs.len(), 3);
    // Account state index is increased
    selected.outputs.iter().for_each(|output| {
        if let Output::Basic(_basic_output) = &output {
            // Basic output remainder has the remaining native tokens
            // TODO reenable when ISA supports NTs again
            // assert_eq!(basic_output.native_token().unwrap().amount().as_u32(), 421);
        }
    });
}
