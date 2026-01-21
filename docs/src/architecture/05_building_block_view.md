---
icon: material/toy-brick-search
---

# Building Block View

<!-- See: https://docs.arc42.org/section-5/ -->

## White box Overall System

The catalyst-libs system is organized as a monorepo containing multiple independent but related libraries.

**Motivation**:  
The system provides reusable libraries for Catalyst ecosystem applications. Libraries are organized by functionality and can be used independently or together.

**Contained Building Blocks**:

1. **Blockchain Integration Layer**: Cardano blockchain interaction
2. **Catalyst Types Layer**: Catalyst-specific data types
3. **Document Management Layer**: Signed document creation and verification
4. **Security Layer**: RBAC and cryptographic operations
5. **Data Processing Layer**: Encoding, decoding, and storage
6. **Voting Layer**: Vote transaction creation and management

**Important Interfaces**:

- **Blockchain Interface**: Node-to-Node protocol, Mithril snapshots
- **Cryptographic Interface**: Ed25519 signing, key derivation
- **Storage Interface**: IPFS, local storage
- **Application Interface**: Rust APIs, FFI, WebAssembly, Flutter/Dart packages

## Level 1: Building Blocks

### Blockchain Integration Layer

**Purpose/Responsibility**:  
Provides functionality to read and follow the Cardano blockchain, decode blocks and transactions, and interact with Cardano nodes.

**Building Blocks**:
- `cardano-chain-follower` (v0.0.19): Chain synchronization and block reading
- `cardano-blockchain-types` (v0.0.9): Common Cardano blockchain data types

**Interfaces**:
- Chain update stream (async channel)
- Block reading API
- Network configuration (mainnet, preprod, preview)

**Directory Location**: `rust/cardano-chain-follower/`, `rust/cardano-blockchain-types/`

**Dependencies**: Pallas, Mithril client, Tokio

**Fulfilled Requirements**: Blockchain synchronization, block reading, multiple network support

### Catalyst Types Layer

**Purpose/Responsibility**:  
Provides Catalyst-specific data types and structures used across the ecosystem.

**Building Blocks**:
- `catalyst-types` (v0.0.14): Core Catalyst types
- `catalyst-voting` (v0.0.2): Voting-related types and protocols
- `catalyst-contest` (v0.0.1): Contest and ballot management

**Interfaces**:
- Type definitions and serialization
- Protocol implementations
- Validation functions

**Directory Location**: `rust/catalyst-types/`, `rust/catalyst-voting/`, `rust/catalyst-contest/`

**Dependencies**: cardano-blockchain-types, ed25519-dalek

**Fulfilled Requirements**: Catalyst type definitions, voting protocols, contest management

### Document Management Layer

**Purpose/Responsibility**:  
Handles creation, signing, verification, and management of Catalyst signed documents.

**Building Blocks**:
- `catalyst-signed-doc` (v0.0.11): Main signed document implementation
- `catalyst-signed-doc-spec` (v0.2.5): Specification and CDDL schemas
- `catalyst-signed-doc-macro` (v0.0.1): Procedural macros for document types

**Interfaces**:
- Document creation API
- Signing and verification API
- Document encoding/decoding (COSE)
- Metadata management

**Directory Location**: `rust/signed_doc/`, `rust/catalyst-signed-doc-spec/`, `rust/catalyst-signed-doc-macro/`

**Dependencies**: catalyst-types, cbork-utils, COSE libraries

**Fulfilled Requirements**: Signed document creation, versioning, collaboration, multiple document types

### Security Layer

**Purpose/Responsibility**:  
Provides Role-Based Access Control (RBAC) registration, key management, and certificate handling.

**Building Blocks**:
- `rbac-registration` (v0.0.15): RBAC registration on Cardano blockchain
- `c509-certificate` (v0.0.3): C509 certificate implementation

**Interfaces**:
- RBAC registration API
- Key derivation API
- Certificate parsing and validation
- Catalyst ID URI handling

**Directory Location**: `rust/rbac-registration/`, `rust/c509-certificate/`

**Dependencies**: cardano-blockchain-types, ed25519-dalek, CBOR libraries

**Fulfilled Requirements**: RBAC registration, Catalyst ID URI, key derivation, certificate handling

### Data Processing Layer

**Purpose/Responsibility**:  
Provides data encoding/decoding, IPFS integration, and immutable ledger operations.

**Building Blocks**:
- `cbork` (v0.0.3): Core CBOR operations
- `cbork-abnf-parser` (v0.0.3): ABNF parser for CBOR
- `cbork-cddl-parser` (v0.0.3): CDDL parser for CBOR
- `cbork-utils` (v0.0.4): CBOR utilities
- `hermes-ipfs` (v0.0.12): IPFS integration
- `immutable-ledger` (v0.2.0): Immutable ledger operations

**Interfaces**:
- CBOR encoding/decoding
- CDDL schema validation
- IPFS storage/retrieval
- Ledger operations

**Directory Location**: `rust/cbork*/`, `rust/hermes-ipfs/`, `rust/immutable-ledger/`

**Dependencies**: CBOR libraries, IPFS libraries

**Fulfilled Requirements**: Data encoding/decoding, IPFS integration, ledger operations

### Voting Layer

**Purpose/Responsibility**:  
Handles creation of vote transactions for Cardano blockchain submission.

**Building Blocks**:
- `vote-tx-v1` (v0.0.1): Vote transaction format v1
- `vote-tx-v2` (v0.1.0): Vote transaction format v2

**Interfaces**:
- Vote transaction creation API
- Transaction encoding
- Format conversion

**Directory Location**: `rust/vote-tx-v1/`, `rust/vote-tx-v2/`

**Dependencies**: cardano-blockchain-types, catalyst-types

**Fulfilled Requirements**: Vote transaction creation, multiple format support

### Python Integration Layer

**Purpose/Responsibility**:  
Provides Python bindings for catalyst-libs Rust crates via Foreign Function Interface (FFI).

**Building Blocks**:
- `catalyst-python`: Python package with FFI bindings to Rust crates

**Interfaces**:
- Python API wrapping Rust crates
- FFI bindings for core functionality
- Python package distribution

**Directory Location**: `catalyst-python/`

**Dependencies**: Rust crates (via FFI), Python 3.x

**Fulfilled Requirements**: Python integration, cross-language support

**Key Features**:
- RBAC operations from Python
- Catalyst Signed Documents from Python
- Catalyst API integration
- Type-safe Python interfaces

## Level 2: Detailed Building Blocks

### cardano-chain-follower

**Purpose**:  
Reads and follows Cardano blockchain updates using Node-to-Node protocol or Mithril snapshots.

**Key Features**:
- Chain synchronization from tip or genesis
- Block reading (single or range)
- Rollback handling
- Mithril snapshot support
- Multiple network support (mainnet, preprod, preview)

**Interfaces**:
- `ChainFollower`: Main follower interface
- `ChainUpdate`: Async stream of chain updates
- `BlockReader`: Block reading interface

**Performance Characteristics**:
- Efficient memory usage
- Async I/O for network operations
- Optional read-ahead queue (planned optimization)

**Directory**: `rust/cardano-chain-follower/`

**Dependencies**: cardano-blockchain-types, mithril-client, tokio, pallas

**Open Issues**: Read-ahead queue optimization for immutable follower (see README)

### cardano-blockchain-types

**Purpose**:  
Common Cardano blockchain data types used across catalyst-libs.

**Key Features**:
- Block structures
- Transaction types
- UTXO types
- Metadata types
- Protocol version support

**Interfaces**:
- Type definitions
- Serialization/deserialization
- Validation functions

**Directory**: `rust/cardano-blockchain-types/`

**Dependencies**: CBOR libraries, protocol libraries

**Fulfilled Requirements**: Common type definitions for Cardano blockchain

### catalyst-signed-doc

**Purpose**:  
Implements Catalyst signed document specification with COSE encoding.

**Key Features**:
- Document creation and signing
- COSE encoding/decoding
- Metadata management
- Versioning support
- Collaboration features

**Interfaces**:
- Document builder API
- Signing API
- Verification API
- Metadata API

**Directory**: `rust/signed_doc/`

**Dependencies**: catalyst-types, catalyst-signed-doc-spec, catalyst-signed-doc-macro, cbork-utils

**Fulfilled Requirements**: Signed document creation, signing, verification, versioning

### rbac-registration

**Purpose**:  
Handles RBAC registration on Cardano blockchain with Catalyst ID URI support.

**Key Features**:
- RBAC registration transaction creation
- Key derivation following BIP-32/BIP-44
- Catalyst ID URI generation
- Role management

**Interfaces**:
- Registration API
- Key derivation API
- URI generation API

**Directory**: `rust/rbac-registration/`

**Dependencies**: cardano-blockchain-types, ed25519-dalek, CBOR libraries

**Fulfilled Requirements**: RBAC registration, key derivation, Catalyst ID URI

### hermes-ipfs

**Purpose**:  
IPFS integration for decentralized document storage.

**Key Features**:
- IPFS node connection
- Document storage and retrieval
- Content addressing
- Network integration

**Interfaces**:
- Storage API
- Retrieval API
- Network configuration

**Directory**: `rust/hermes-ipfs/`

**Dependencies**: IPFS libraries

**Fulfilled Requirements**: IPFS integration for document storage

### catalyst-python

**Purpose**:  
Python bindings for catalyst-libs Rust crates via FFI.

**Key Features**:
- Python API for RBAC operations
- Python API for Catalyst Signed Documents
- Python API for Catalyst API integration
- Type-safe Python interfaces

**Interfaces**:
- Python package API
- FFI bindings to Rust crates
- Python type definitions

**Directory**: `catalyst-python/`

**Dependencies**: Rust crates (via FFI), Python 3.x

**Fulfilled Requirements**: Python integration, cross-language support

## Integration with External Systems

### cat-gateway Integration

The `cat-gateway` service (in catalyst-voices repository) uses multiple catalyst-libs crates:

- `cardano-chain-follower`: For blockchain synchronization
- `rbac-registration`: For RBAC handling
- `catalyst-signed-doc`: For document processing
- `c509-certificate`: For certificate validation
- `catalyst-types`: For type definitions

This demonstrates the reusability of catalyst-libs across different applications.

### Catalyst Voices Integration

The Catalyst Voices Flutter application uses:

- Flutter/Dart packages from catalyst-libs
- Rust crates via FFI or platform channels
- Shared types and protocols

## Building Block Dependencies

```
cardano-blockchain-types (foundation)
    ↑
    ├── cardano-chain-follower
    ├── catalyst-types
    ├── rbac-registration
    └── vote-tx-v1, vote-tx-v2

catalyst-types
    ↑
    ├── catalyst-signed-doc
    ├── catalyst-voting
    └── catalyst-contest

catalyst-signed-doc-spec
    ↑
    └── catalyst-signed-doc

cbork-utils
    ↑
    └── catalyst-signed-doc
```

This dependency structure ensures:
- Clear separation of concerns
- Minimal dependencies
- Reusability of foundational types
- Independent versioning where possible
