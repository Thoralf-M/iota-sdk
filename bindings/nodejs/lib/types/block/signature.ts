// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { plainToInstance, Type } from 'class-transformer';
import { HexEncodedString } from '../utils';
import {
    Ed25519PublicKey,
    PublicKey,
    PublicKeyDiscriminator,
} from './public-key';

/**
 * All of the signature types.
 */
enum SignatureType {
    Ed25519 = 0,
}

abstract class Signature {
    private type: SignatureType;

    constructor(type: SignatureType) {
        this.type = type;
    }

    /**
     * The type of signature.
     */
    getType(): SignatureType {
        return this.type;
    }

    public static parse(data: any): Signature {
        if (data.type == SignatureType.Ed25519) {
            return plainToInstance(
                Ed25519Signature,
                data,
            ) as any as Ed25519Signature;
        }
        throw new Error('Invalid JSON');
    }
}

/**
 * Ed25519Signature signature.
 */
class Ed25519Signature extends Signature {
    /**
     * The public key.
     */
    publicKey: Ed25519PublicKey;
    /**
     * The signature.
     */
    signature: HexEncodedString;

    constructor(publicKey: HexEncodedString, signature: HexEncodedString) {
        super(SignatureType.Ed25519);
        this.publicKey = new Ed25519PublicKey(publicKey);
        this.signature = signature;
    }

    /**
     * The hex encoded Ed25519 public key.
     */
    ed25519PublicKey(): HexEncodedString {
        return this.publicKey.publicKey;
    }
}

export { SignatureType, Ed25519Signature, Signature };
