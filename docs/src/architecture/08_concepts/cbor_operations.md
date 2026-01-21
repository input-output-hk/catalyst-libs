# CBOR Operations

## Overview

Catalyst-libs includes a family of CBOR (Concise Binary Object Representation) operation crates for encoding, decoding, and processing CBOR data.

## Purpose

CBOR operations are essential for:
- Blockchain data encoding/decoding
- Protocol message serialization
- Efficient binary data representation
- CDDL schema validation

## CBOR Crate Family

### cbork (v0.0.3)

Core CBOR operations crate providing fundamental CBOR functionality.

**Features**:
- CBOR encoding/decoding
- Basic CBOR operations
- Type conversions

### cbork-utils (v0.0.4)

Utility functions and helpers for CBOR operations.

**Features**:
- Common CBOR utilities
- Helper functions
- Convenience methods

### cbork-abnf-parser (v0.0.3)

ABNF (Augmented Backus-Naur Form) parser for CBOR schemas.

**Features**:
- ABNF grammar parsing
- Schema validation
- CBOR structure validation

### cbork-cddl-parser (v0.0.3)

CDDL (Concise Data Definition Language) parser for CBOR.

**Features**:
- CDDL schema parsing
- Schema validation
- Type checking

## Usage in Catalyst-Libs

CBOR operations are used throughout catalyst-libs:

- **Blockchain Types**: Cardano data is encoded in CBOR
- **Signed Documents**: COSE uses CBOR encoding
- **Transactions**: Vote transactions use CBOR
- **RBAC**: Registration data uses CBOR

## CDDL Schemas

Many Catalyst specifications use CDDL to define data structures:

- Signed document schemas
- Transaction formats
- Protocol messages
- Metadata structures

The `cbork-cddl-parser` enables validation against these schemas.

## Integration

CBOR crates are dependencies for:
- `catalyst-signed-doc`: Uses CBOR for COSE encoding
- `cardano-blockchain-types`: Uses CBOR for blockchain data
- `rbac-registration`: Uses CBOR for registration data
- `vote-tx-v1` and `vote-tx-v2`: Use CBOR for transactions

## Performance

CBOR provides:
- **Efficiency**: Compact binary representation
- **Speed**: Fast encoding/decoding
- **Compatibility**: Standard format for Cardano

## Validation

CDDL and ABNF parsers enable:
- Schema validation before encoding
- Type checking
- Structure verification
- Protocol compliance

See `rust/cbork*/` directories for implementation details.
