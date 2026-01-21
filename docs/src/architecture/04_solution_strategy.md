---
icon: material/strategy
---

# Solution Strategy

<!-- See: https://docs.arc42.org/section-4/ -->

## Technology Decisions

### Rust as Core Language

**Decision**: Use Rust for all core libraries

**Rationale**:
- Performance and memory safety for blockchain operations
- Strong type system for protocol correctness
- Excellent WebAssembly support
- Growing ecosystem for Cardano development
- Can be used from multiple languages via FFI

**Consequences**:
- High performance and safety
- Learning curve for developers
- Strict compilation and linting requirements

### Monorepo Structure

**Decision**: Single repository for all libraries

**Rationale**:
- Simplified dependency management
- Shared tooling and configuration
- Easier cross-crate refactoring
- Coordinated versioning
- Single CI/CD pipeline

**Consequences**:
- Large repository size
- Need for clear crate boundaries
- Coordinated releases

### Modular Crate Design

**Decision**: Separate crates for distinct functionalities

**Rationale**:
- Reusability: Use only what you need
- Independent versioning
- Clear separation of concerns
- Easier testing and maintenance

**Crate Categories**:
1. **Blockchain**: cardano-chain-follower, cardano-blockchain-types
2. **Catalyst Types**: catalyst-types, catalyst-voting, catalyst-contest
3. **Documents**: signed_doc, catalyst-signed-doc-spec, catalyst-signed-doc-macro
4. **Security**: rbac-registration, c509-certificate
5. **Data Processing**: cbork family, hermes-ipfs, immutable-ledger
6. **Voting**: vote-tx-v1, vote-tx-v2

### Protocol-First Approach

**Decision**: Implement Cardano Improvement Proposals (CIPs) and Catalyst specifications

**Rationale**:
- Standards compliance
- Interoperability
- Clear specifications
- Community alignment

**Specifications**:
- CIP-30: Wallet integration
- CIP-36: Voting registration
- CIP-95: Message signing
- Catalyst Signed Document specification
- Catalyst ID URI specification

### Cross-Platform Support

**Decision**: Support multiple platforms and languages

**Rationale**:
- Broader adoption
- Platform-specific optimizations
- Language ecosystem integration

**Approaches**:
- **Rust**: Native crates
- **WebAssembly**: wasm32 target for web
- **Flutter/Dart**: Dart packages wrapping Rust or native implementations
- **Python**: FFI bindings via catalyst-python

## Architectural Patterns

### Library-First Design

- Libraries are independent and reusable
- Applications consume libraries, not the reverse
- Clear API boundaries
- Minimal dependencies between crates

### Error Handling

- Comprehensive error types using `thiserror`
- No panics in library code (deny panic)
- Proper error propagation
- Clear error messages

### Type Safety

- Strong typing for protocol data
- CDDL specifications for data structures
- Compile-time validation where possible
- Runtime validation for external data

### Async/Await

- Tokio for async runtime
- Async interfaces for I/O operations
- Efficient resource usage
- Support for concurrent operations

## Design Principles

1. **Correctness First**: Protocol compliance and type safety
2. **Performance**: Efficient resource usage
3. **Security**: Secure cryptographic operations
4. **Documentation**: Comprehensive and up-to-date
5. **Testing**: High coverage with integration tests
6. **Maintainability**: Clear structure and conventions

## Reusability Strategy

### Standalone Usage

- Each crate can be used independently
- Minimal required dependencies
- Clear feature flags for optional functionality

### Integration Patterns

- **Direct Rust**: Use crates directly
- **FFI**: Foreign Function Interface for other languages
- **WebAssembly**: Compile to wasm32 for web
- **Flutter**: Dart packages or platform channels
- **Python**: FFI bindings

### Versioning Strategy

- Semantic versioning per crate
- Git tags for releases
- Backward compatibility where possible
- Clear migration paths for breaking changes
