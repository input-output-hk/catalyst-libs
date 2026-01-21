---
icon: material/telescope
---

# System Scope and Context

<!-- See: https://docs.arc42.org/section-3/ -->

## Business Context

Catalyst-libs provides foundational libraries for the Catalyst ecosystem, enabling developers to build applications for Cardano governance, voting, and proposal management.

### External Business Entities

- **Catalyst Community**: End users who participate in Catalyst governance
- **Application Developers**: Developers building Catalyst applications (e.g., Catalyst Voices)
- **Catalyst Team**: Maintainers and contributors to the libraries
- **IOHK**: Infrastructure and blockchain provider

### Business Interfaces

- **Library Consumers**: Applications using catalyst-libs (e.g., Catalyst Voices, cat-gateway)
- **Blockchain Networks**: Cardano mainnet, preprod, preview
- **IPFS Network**: For decentralized document storage
- **Mithril Network**: For efficient blockchain snapshots

## Technical Context

### External Systems and Interfaces

#### Cardano Blockchain

- **Protocol**: Node-to-Node protocol for chain synchronization
- **Networks**: Mainnet, Preprod, Preview
- **Data**: Blocks, transactions, UTXOs, metadata
- **Interface**: Direct node connection or Mithril snapshots

#### Mithril

- **Purpose**: Efficient blockchain snapshot access
- **Interface**: Mithril client library
- **Usage**: Alternative to full node synchronization

#### IPFS (via hermes-ipfs)

- **Purpose**: Decentralized document storage
- **Interface**: IPFS network protocol
- **Usage**: Storing and retrieving signed documents

#### Catalyst Applications

- **Catalyst Voices**: Flutter application using catalyst-libs
- **cat-gateway**: Backend service using catalyst-libs crates
- **Other Applications**: Any application integrating catalyst-libs

### Technical Interfaces

#### Rust Crates

- **Public APIs**: Library interfaces for Rust applications
- **FFI**: Foreign Function Interface for non-Rust languages
- **WebAssembly**: wasm32 target for web applications

#### Flutter/Dart Packages

- **Pub Package**: Published to pub.dev or git
- **Platforms**: iOS, Android, Web, Desktop

#### Python Bindings

- **FFI**: Python bindings via catalyst-python
- **Interface**: Python API wrapping Rust crates

### Dependencies

#### External Rust Crates

- **Pallas**: Cardano protocol implementation
- **Mithril Client**: Blockchain snapshot access
- **ed25519-dalek**: Cryptographic signing
- **CBOR libraries**: Data encoding/decoding
- **IPFS libraries**: Decentralized storage

#### Development Tools

- **Earthly**: Build automation
- **MkDocs**: Documentation generation
- **Testing frameworks**: Unit and integration tests

## Mapping Input/Output to Channels

### Input Channels

1. **Blockchain Data**: From Cardano nodes or Mithril snapshots
2. **User Actions**: Via application APIs (signing, voting, document creation)
3. **Configuration**: Network selection, node endpoints, keys

### Output Channels

1. **Processed Data**: Decoded blocks, transactions, documents
2. **Signed Documents**: COSE-encoded documents ready for storage
3. **Vote Transactions**: Transaction data ready for blockchain submission
4. **API Responses**: Structured data for application consumption

### Data Flow

- **Blockchain → Libraries**: Chain updates, blocks, transactions
- **Libraries → Applications**: Processed data, signed documents, vote transactions
- **Applications → Libraries**: Signing requests, document creation, queries
- **Libraries → Blockchain**: Vote transactions, RBAC registrations
