/* istanbul ignore file */
/* tslint:disable */
/* eslint-disable */

import type { TransactionSignature_Ed25519Signature } from './TransactionSignature_Ed25519Signature';
import type { TransactionSignature_FeePayerSignature } from './TransactionSignature_FeePayerSignature';
import type { TransactionSignature_MultiAgentSignature } from './TransactionSignature_MultiAgentSignature';
import type { TransactionSignature_MultiEd25519Signature } from './TransactionSignature_MultiEd25519Signature';
import type { TransactionSignature_Secp256k1EcdsaSignature } from './TransactionSignature_Secp256k1EcdsaSignature';

/**
 * An enum representing the different transaction signatures available
 */
export type TransactionSignature = (TransactionSignature_Ed25519Signature | TransactionSignature_MultiEd25519Signature | TransactionSignature_MultiAgentSignature | TransactionSignature_FeePayerSignature | TransactionSignature_Secp256k1EcdsaSignature);

