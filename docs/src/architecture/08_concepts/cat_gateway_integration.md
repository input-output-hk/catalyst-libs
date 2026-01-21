# cat-gateway Integration Patterns

## Overview

This document describes how the `cat-gateway` service (Catalyst Data Gateway) integrates with and uses catalyst-libs crates.

## cat-gateway Service

The cat-gateway is a backend service that provides:
- REST API for Catalyst data
- Blockchain synchronization
- Document storage and retrieval
- RBAC handling
- Database indexing

**Location**: `catalyst-voices/catalyst-gateway/` (separate repository)

## Catalyst-Libs Crates Used

### cardano-chain-follower

**Usage**: Blockchain synchronization

- Follows Cardano blockchain updates
- Processes blocks for RBAC registrations
- Indexes relevant transactions
- Provides chain data to API

**Integration Pattern**:
```rust
// cat-gateway uses chain follower to sync blockchain
let follower = ChainFollower::new(config);
// Process chain updates
```

### rbac-registration

**Usage**: RBAC key lookup and validation

- Looks up RBAC registrations on blockchain
- Validates Catalyst ID URIs
- Verifies role permissions
- Enforces access control

**Integration Pattern**:
```rust
// cat-gateway uses rbac-registration for key lookup
let registration = lookup_rbac_registration(id_uri)?;
// Verify permissions
```

### catalyst-signed-doc

**Usage**: Document processing

- Verifies signed documents
- Processes document metadata
- Handles document storage
- Validates signatures

**Integration Pattern**:
```rust
// cat-gateway uses signed-doc for verification
let doc = verify_signed_document(cose_data)?;
// Process document
```

### c509-certificate

**Usage**: Certificate validation

- Parses C509 certificates
- Validates certificate signatures
- Extracts role information
- Verifies against on-chain data

**Integration Pattern**:
```rust
// cat-gateway uses c509-certificate for validation
let cert = parse_c509_certificate(cert_data)?;
// Validate and extract roles
```

### catalyst-types

**Usage**: Type definitions

- Uses Catalyst types throughout
- Ensures type consistency
- Shares types with API responses
- Type-safe data handling

## Integration Architecture

### Data Flow

1. **Blockchain Sync**: `cardano-chain-follower` → Database
2. **Document Processing**: API → `catalyst-signed-doc` → Database
3. **RBAC Validation**: API → `rbac-registration` → Blockchain lookup
4. **API Responses**: Database → `catalyst-types` → JSON API

### Database Layer

cat-gateway uses:
- **PostgreSQL (event-db)**: Event and document storage
- **ScyllaDB (index-db)**: Indexing and queries

catalyst-libs crates work with these databases through cat-gateway's database layer.

## API Integration

cat-gateway exposes REST APIs that:
- Use catalyst-libs types in responses
- Process data using catalyst-libs functions
- Validate using catalyst-libs validators
- Encode/decode using catalyst-libs serialization

## Best Practices

### Error Handling

- Use catalyst-libs error types
- Propagate errors appropriately
- Provide clear error messages

### Type Safety

- Use catalyst-types throughout
- Avoid manual type conversions
- Leverage type system

### Performance

- Cache frequently accessed data
- Optimize database queries
- Use async operations

## Example Integration

```rust
// cat-gateway service using catalyst-libs
use cardano_chain_follower::ChainFollower;
use rbac_registration::lookup_registration;
use catalyst_signed_doc::verify_document;

// Sync blockchain
let updates = follower.get_updates().await?;

// Process documents
let doc = verify_document(document_data)?;

// Check permissions
let registration = lookup_registration(catalyst_id)?;
```

## Benefits of Integration

- **Reusability**: Shared libraries across services
- **Consistency**: Same types and logic everywhere
- **Maintainability**: Single source of truth
- **Testing**: Libraries tested independently

## See Also

- [Building Block View](../05_building_block_view.md) - Overall system structure
- [Runtime View](../06_runtime_view.md) - Runtime scenario 6 for cat-gateway
- [Deployment View](../07_deployment_view.md) - cat-gateway deployment
