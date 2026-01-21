---
Title: ED25519 Document Signature Keys Derivation
Authors:
    - Steven Johnson <steven.johnson@iohk.io>
Created: 2024-11-29
License: [CC-BY-4.0]
---

## Abstract

Defines how Document Signature Keys are derived using
[Catalyst HD Key Derivation](./hd-key-derivation.md).

## Motivation: why is this CIP necessary?

Users in Catalyst are required to sign various documents with various authorities.
This is used as a way to authenticate not just the user acted to sign the document, but that they
knowingly acted in the capacity of the role they are registered under.

This helps clearly delineate actions, and also helps with organizational keys where certain
parties may be trusted with a derived key for one role, but not others.

For example, an organization may internally delegate writing and submitting of proposals to one person,
but they do not also want to give that person the capability to vote on a proposal.

This scheme allows for that segregation of roles and responsibilities.

## Specification

For reference, see [Catalyst HD Key Derivation](./hd-key-derivation.md).
This document defines how ED25519 document signing keys are derived from the master seed phrase.

### `usage'`

The ED25519 private signing key is derived with `usage'` set to 0.

### `role`

Role maps 1:1 to the role the user will be under when using the key, and this maps to their on-chain registration.
The registered public key for the Role, MUST match the derived key or documents will not be accepted as
valid.

### `index`

Index maps 1:1 to the key rotation currently used for the role, and this maps to their on-chain registration.
The registered public key for the Role+Rotation, MUST match the derived key or documents will not be accepted as
valid.

## Usage

Having derived the Private signing key, the public key can be obtained and posted on chain in an RBAC registration for the role.
The private key can then be used to authoritatively sign documents for that registration under that role.

## Reference Implementation

The first implementation was Catalyst Voices. This specification is now implemented in the `rbac-registration` crate and used by multiple applications.

*TODO: Generate a set of test vectors which conform to this specification.*

## Rationale: how does this CIP achieve its goals?

By leveraging known working Key Derivation techniques and simply modifying the path we inherit the properties of those methods.

## Path to Active

### Acceptance Criteria

Working Implementation before Fund 14.

### Implementation Plan

Fund 14 project catalyst deployed this scheme for Key derivation.>

## Copyright

This document is licensed under [CC-BY-4.0]

[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
