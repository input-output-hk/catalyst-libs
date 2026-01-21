# IPFS Integration

## Overview

The `hermes-ipfs` crate provides IPFS (InterPlanetary File System) integration for decentralized document storage in the Catalyst ecosystem.

## Purpose

IPFS integration enables:
- Decentralized storage of signed documents
- Content-addressable document retrieval
- Distributed document sharing
- Resilience against single points of failure

## Crate Details

**Crate**: `hermes-ipfs` (v0.0.12)

### Features

- IPFS node connection and management
- Document storage (add to IPFS)
- Document retrieval (get from IPFS)
- Content addressing (CID generation)
- Network integration

## Usage Patterns

### Storing Documents

Signed documents created with `catalyst-signed-doc` can be stored in IPFS:

1. Create and sign document using `catalyst-signed-doc`
2. Store document in IPFS using `hermes-ipfs`
3. Receive Content Identifier (CID) for retrieval
4. Store CID reference separately (e.g., in database)

### Retrieving Documents

Documents can be retrieved from IPFS using their CID:

1. Use CID to retrieve document from IPFS
2. Verify document using `catalyst-signed-doc`
3. Process document content

## Integration with Signed Documents

The IPFS integration works seamlessly with Catalyst signed documents:

- Documents are stored as-is (COSE format)
- CID provides immutable reference
- Documents can be verified after retrieval
- Version history can be maintained via CIDs

## Network Options

### Local IPFS Node

- Run IPFS node locally
- Full control over storage
- Direct network participation

### IPFS Gateway

- Use public or private IPFS gateway
- No local node required
- Simpler setup

### IPFS Network

- Participate in IPFS network
- Contribute to document availability
- Distributed storage

## Reliability Considerations

- **Network Availability**: IPFS network reliability affects document access
- **Pinning**: Important documents should be pinned to ensure availability
- **Fallback**: Consider local caching or alternative storage backends
- **Redundancy**: Multiple IPFS nodes can store same document

## Security

- Documents are stored as-is (encrypted if document itself is encrypted)
- CID provides content integrity verification
- Access control is handled at application level, not IPFS level

## Implementation Notes

The `hermes-ipfs` crate uses a fork of rust-ipfs libraries, customized for Catalyst needs.

See `rust/hermes-ipfs/` for implementation details.
