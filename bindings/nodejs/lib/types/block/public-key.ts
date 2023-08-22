// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

// Temp solution for not double parsing JSON
import { HexEncodedString } from '../utils';

/**
 * All of the public key types.
 */
enum PublicKeyType {
    Ed25519 = 0,
}

/**
 * A public key.
 */
abstract class PublicKey {
    readonly type: PublicKeyType;

    constructor(type: PublicKeyType) {
        this.type = type;
    }
}

/**
 * Ed25519 public key.
 */
class Ed25519PublicKey extends PublicKey {
    /**
     * The public key.
     */
    publicKey: HexEncodedString;

    constructor(publicKey: HexEncodedString) {
        super(PublicKeyType.Ed25519);
        this.publicKey = publicKey;
    }
}

export { Ed25519PublicKey, PublicKey, PublicKeyType };
