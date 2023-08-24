// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { plainToInstance } from 'class-transformer';
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
    public static parse(data: any): PublicKey {
        if (data.type == PublicKeyType.Ed25519) {
            return plainToInstance(
                Ed25519PublicKey,
                data,
            ) as any as Ed25519PublicKey;
        }
        throw new Error('Invalid JSON');
    }
}

/**
 * Ed25519 public key.
 */
class Ed25519PublicKey extends PublicKey {
    /**
     * The public key.
     */
    readonly publicKey: HexEncodedString;

    constructor(publicKey: HexEncodedString) {
        super(PublicKeyType.Ed25519);
        this.publicKey = publicKey;
    }
}

const PublicKeyDiscriminator = {
    property: 'type',
    subTypes: [{ value: Ed25519PublicKey, name: PublicKeyType.Ed25519 as any }],
};

export { PublicKeyDiscriminator, Ed25519PublicKey, PublicKey, PublicKeyType };
