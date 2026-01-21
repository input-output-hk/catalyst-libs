# Voting Transaction Formats

## Overview

Catalyst-libs provides two vote transaction formats for submitting votes to the Cardano blockchain:
- **vote-tx-v1**: Original vote transaction format
- **vote-tx-v2**: Updated vote transaction format with enhanced features

## Vote Transaction v1

**Crate**: `vote-tx-v1` (v0.0.1)

### Purpose

The v1 format provides the original vote transaction structure for Catalyst voting.

### Features

- Basic vote transaction encoding
- Support for proposal voting
- CBOR encoding for blockchain submission

### Usage

```rust
use vote_tx_v1::*;
// Build and encode vote transaction
```

## Vote Transaction v2

**Crate**: `vote-tx-v2` (v0.1.0)

### Purpose

The v2 format provides an enhanced vote transaction structure with additional features and improved efficiency.

### Features

- Enhanced vote transaction encoding
- Support for additional vote types
- Improved efficiency and validation
- Backward compatibility considerations

### Usage

```rust
use vote_tx_v2::*;
// Build and encode vote transaction
```

## Format Selection

Applications should choose the format based on:
- **Compatibility**: Which format is supported by the current Catalyst protocol
- **Features**: Which format provides required features
- **Migration**: If migrating from v1 to v2, consider migration path

## Integration with Catalyst Types

Both formats use types from `catalyst-types` for:
- Proposal identifiers
- Vote choices
- Voting power information
- Metadata

## Blockchain Submission

Vote transactions are encoded to CBOR and can be:
- Submitted directly to Cardano node
- Integrated with wallet (CIP-30)
- Included in transaction batches

## Version Compatibility

- **v1**: Legacy format, may be deprecated in future
- **v2**: Current recommended format

See `rust/vote-tx-v1/` and `rust/vote-tx-v2/` for implementation details.
