# Catalyst Types

## Overview

The `catalyst-types` crate provides core Catalyst-specific data types and structures used across the Catalyst ecosystem.

## Purpose

Catalyst Types serves as a foundational crate that defines common types, enums, and structures specific to the Catalyst governance system. These types are used by other catalyst-libs crates and by applications building on Catalyst.

## Key Types

### Core Types

- **Catalyst ID**: Identifier for Catalyst entities
- **Event Types**: Types for Catalyst events and voting rounds
- **Proposal Types**: Types related to proposals and voting
- **Role Types**: Types for RBAC roles and permissions

### Serialization

All types support:
- **CBOR encoding/decoding**: For blockchain and network transmission
- **JSON serialization**: For API interfaces
- **Type validation**: Compile-time and runtime validation

## Usage

The `catalyst-types` crate is a dependency for:
- `catalyst-signed-doc`: Uses types for document metadata
- `catalyst-voting`: Uses types for voting protocols
- `catalyst-contest`: Uses types for contest management
- `vote-tx-v1` and `vote-tx-v2`: Use types for vote transactions

## Integration

Applications using catalyst-libs typically import `catalyst-types` to:
- Work with Catalyst-specific data structures
- Ensure type safety across the ecosystem
- Share types between different parts of an application

## Version

Current version: **v0.0.14**

See `rust/catalyst-types/` for implementation details.
