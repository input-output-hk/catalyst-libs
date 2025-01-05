---
Title: RBAC Key Identifier URI Specification
Category: Catalyst
Status: Proposed
Authors:
    - Steven Johnson <steven.johnson@iohk.io>
Implementors:
    - Catalyst Fund 14
Discussions: []
Created: 2025-01-05
License: CC-BY-4.0
---

* [Abstract](#abstract)
* [Motivation: why is this CIP necessary?](#motivation-why-is-this-cip-necessary)
* [Specification](#specification)
  * [URI](#uri)
  * [`scheme`](#scheme)
  * [`authority`](#authority)
    * [`authority` - `host`](#authority---host)
      * [List of defined hosts](#list-of-defined-hosts)
    * [`authority` - `userinfo`](#authority---userinfo)
      * [Lists of defined subnetwork `userinfo` values](#lists-of-defined-subnetwork-userinfo-values)
        * [Cardano](#cardano)
  * [`path`](#path)
* [Reference Implementation](#reference-implementation)
* [Test Vectors](#test-vectors)
* [Rationale: how does this CIP achieve its goals?](#rationale-how-does-this-cip-achieve-its-goals)
* [Path to Active](#path-to-active)
  * [Acceptance Criteria](#acceptance-criteria)
  * [Implementation Plan](#implementation-plan)
* [Copyright](#copyright)

## Abstract

Definition of a [URI] which allows for RBAC keys used for different purposes to be easily and
unambiguously identified.

## Motivation: why is this CIP necessary?

There is a need to identify which Key from a RBAC registration was used to sign data.
RBAC defines a universal keychain of different keys that can be used for different purposes.
They can be used not only for Signatures, but also Encryption.

Therefore, there needs to be an unambiguous and easy to lookup identifier to signify which key was
used for a particular purpose.

This document defines a [URI] scheme to unambiguously define a particular key with reference to a
particular RBAC keychain.

## Specification

### URI

The RBAC Kid is formatted using a [Universal Resource Identifier].
Refer to [RFC3986] for the specification of the URI format.

### `scheme`

The [scheme](https://datatracker.ietf.org/doc/html/rfc3986#section-3.1) **MUST** be `kid.catalyst-rbac`;

### `authority`

The [authority](https://datatracker.ietf.org/doc/html/rfc3986#section-3.2) references the blockchain or network
the key was registered within.

It is perfectly valid for a Kid to reference a different network than the place where the Key is used.
For example, a `cardano` KID can be used to post documents to `IPFS`.
Its purpose is to define WHERE the key was registered, and nothing more.

The Authority will consist of a `host` and optional `userinfo`.

#### `authority` - `host`

The [host](https://datatracker.ietf.org/doc/html/rfc3986#section-3.2.2)
refers to the network type where the RBAC registration was made.
It **IS NOT** resolvable with **DNS**, and **IS NOT** a public host name.
It is used as a decentralized network identifier.
The consumer of the `KID` must be able to resolve these host names.

##### List of defined hosts

| `host` | Description |
| --- | --- |
| `cardano` | Cardano Blockchain |
| `midnight` | Midnight Blockchain |
| `ethereum` | Ethereum Blockchain |
| `cosmos` | Cosmos Blockchain |

#### `authority` - `userinfo`

The [userinfo](https://datatracker.ietf.org/doc/html/rfc3986#section-3.2.1)
is used to distinguish a subnetwork from the primary main network.
The absence of `userinfo` is used to indicate the primary main network.

##### Lists of defined subnetwork `userinfo` values

###### Cardano

| `userinfo` | Description |
| --- | --- |
| `preprod` | Cardano Pre-Production Network |
| `preview` | Cardano Preview Network |
| 0x<hex_number>  | Cardano network identified by this magic number in hex |

### `path`

The [path](https://datatracker.ietf.org/doc/html/rfc3986#section-3.3) defines the actual key within the registration.
Keys are defined relative to the very first Role0 Key registered in any RBAC registration.

The overall `path` specification is: `<initial role0 key>/<role>/<rotation>#encrypt`

* `<initial role 0 key>` - This is the very first role 0 key used to post the registration to the network.
  * It is the [Base64 URL] encoded binary data of the role 0 public key.
  * This does not change, even if the Initial Role 0 key is revoked.
  * This allows for an unambiguous identifier for the RBAC keychain.
  * It is not necessarily the key being identified.
* `<role>` - This is the Role number being used.
  * It is a positive number, starting at 0, and no greater than 65535.
* `<rotation>` - This is the rotation of the defined role key being identified.
  * It starts at 0 for the first published key for the role, and increments by one for each subsequent published rotation.
  * This number refers to the published sequence of keys for the role in the RBAC registration keychain,
  not the index used in the key derivation.
  * It is positive and no greater than 65535.
* `#encrypt` - [Fragment](https://datatracker.ietf.org/doc/html/rfc3986#section-3.5)
  disambiguates Encryption Public Keys from signing public keys.
  * Roles can have 1 active public signing key, and 1 active public encryption key.
  * By default, the URL is referencing the signing public key.
  * If a public encryption key is being identified, then the fragment `#encrypt` is appended to the [Universal Resource Identifier].

## Reference Implementation

The first implementation will be Catalyst Voices.

## Test Vectors

* `kid.catalyst-rbac://cardano/<key>/0/0`
  * A Signing key registered on the Cardano Main network.
  * Role 0 - Rotation 0.
  In this example, it is exactly the same as the `<key>`.
* `kid.catalyst-rbac://preprod@cardano/ed25519/<key>/7/3`
  * A Signing key registered on the Cardano pre-production network.
  * Role 7 - Rotation 3.
  The Key for Role 7, and its third published rotation
  (i.e., the fourth key published, the first is the initial key, plus 3 rotations following it).
* `kid.catalyst-rbac://preprod@cardano/ed25519/<key>/2/0#encrypt`
  * A Public Encryption key registered on the Cardano pre-production network.
  * Role 2 - Rotation 0.
  The initially published Public Encryption Key for Role 2.
* `kid.catalyst-rbac://midnight/<key>/0/1`
  * A Signing key registered on the Midnight Blockchain Main network
  * Role 0 - Rotation 1.
  In this example, it is NOT the same as the `<key>`, as it identifies the first rotation after `<key>`.
* `kid.catalyst-rbac://midnight/encrypt/<key>/2/1#encrypt`
  * A public encryption key registered on the Midnight Blockchain Main network.
  * Role 2 - Rotation 1.

## Rationale: how does this CIP achieve its goals?

By creating a [URI] to identify keys,
we allow the unambiguous and flexible identification of any RBAC Key that was used for any purpose.

## Path to Active

### Acceptance Criteria

Working Implementation before Fund 14.

### Implementation Plan

Fund 14 project catalyst will deploy this scheme for Key Identification.

## Copyright

This document is licensed under [CC-BY-4.0](https://creativecommons.org/licenses/by/4.0/legalcode).

[URI]: https://datatracker.ietf.org/doc/html/rfc3986
[Universal Resource Identifier]: https://datatracker.ietf.org/doc/html/rfc3986
[RFC3986]: https://datatracker.ietf.org/doc/html/rfc3986
[Base64 URL]: https://datatracker.ietf.org/doc/html/rfc4648#section-5
