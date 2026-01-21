---
icon: material/handcuffs
---

# Architecture Constraints

<!-- See: https://docs.arc42.org/section-2/ -->

## Technical Constraints

### Programming Languages and Versions

- **Rust**: Edition 2024, with strict linting rules (deny warnings, deny missing docs)
- **Flutter/Dart**: Latest stable versions
- **Python**: Python 3.x for FFI bindings

### Cardano Protocol Constraints

- Must support Cardano mainnet, preprod, and preview networks
- Compatibility with Cardano Node-to-Node protocol
- Support for Mithril snapshots for efficient chain synchronization
- Must handle protocol updates and version changes

### Cryptographic Constraints

- Ed25519 for signing (ed25519-dalek)
- COSE (CBOR Object Signing and Encryption) for document signing
- Support for C509 certificates
- Secure key derivation following BIP-32/BIP-44 standards

### Data Format Constraints

- CBOR encoding/decoding for blockchain data
- CDDL (Concise Data Definition Language) for specifications
- JSON for API interfaces
- Base64 URL encoding for Catalyst ID URIs

### Platform Constraints

- Cross-platform support (Linux, macOS, Windows)
- WebAssembly (wasm32) support for web applications
- Mobile platform support via Flutter

## Organizational Constraints

### Development Process

- All code must pass strict linting (clippy pedantic, deny unwrap/expect)
- Comprehensive documentation required (deny missing docs)
- Architecture Decision Records (ADRs) for significant decisions
- Code reviews required for all changes

### Licensing

- Dual licensed: MIT OR Apache-2.0
- All contributions must be compatible with these licenses

### Repository Structure

- Monorepo structure for shared dependencies
- Separate crates/packages for modularity
- Git tags for versioning individual crates

## Conventions

### Code Style

- Rust: rustfmt with workspace configuration
- Deny unwrap/expect - use proper error handling
- Deny todo/unimplemented - complete implementations required
- Comprehensive error types using thiserror

### Documentation

- Arc42 standard for architecture documentation
- API documentation for all public interfaces
- README files for each crate/package
- Examples for common use cases

### Versioning

- Semantic versioning for crates
- Git tags for releases (format: `crate-name/vX.Y.Z`)
- Workspace-level version management

## Known Limitations

### Chain Follower

- Disk I/O performance for immutable follower (see cardano-chain-follower README)
- Read-ahead queue optimization planned but not yet implemented

### IPFS Integration

- Uses fork of rust-ipfs (hermes-ipfs)
- Network reliability depends on IPFS network

### WebAssembly

- Some crates support wasm32, but not all features may be available
- File system operations limited in wasm32 environment
