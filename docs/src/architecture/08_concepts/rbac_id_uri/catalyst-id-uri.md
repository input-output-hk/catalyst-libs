---
Title: RBAC Catalyst Identifier URI Specification
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
      * [Example `userinfo` with a `hostname`](#example-userinfo-with-a-hostname)
  * [`path`](#path)
* [Reference Implementation](#reference-implementation)
* [Test Vectors](#test-vectors)
* [Rationale: how does this CIP achieve its goals?](#rationale-how-does-this-cip-achieve-its-goals)
* [Path to Active](#path-to-active)
  * [Acceptance Criteria](#acceptance-criteria)
  * [Implementation Plan](#implementation-plan)
* [Copyright](#copyright)

## Abstract

Definition of a [URI], which allows for RBAC keys used for different purposes to be easily and
unambiguously identified.

## Motivation: why is this CIP necessary?

There is a need to identify which RBAC Registration is referenced,
or which Key from a RBAC registration was used to sign data.
RBAC defines a universal keychain of different keys that can be used for different purposes.
They can be used not only for Signatures, but also Encryption.

Sometimes all that is required is to identify the individual keychain.
Other times a specific key on the chain needs to be referenced.

Therefore, there needs to be an unambiguous and easy to lookup identifier to signify which keychain,
or key in a particular chain was used for a particular purpose.

This document defines a [URI] scheme to unambiguously define a keychain or a specific key within the keychain.

## Specification

### URI

The Catalyst RBAC ID is formatted using a [Universal Resource Identifier].
Refer to [RFC3986] for the specification of the URI format.

### `scheme`

The [scheme](https://datatracker.ietf.org/doc/html/rfc3986#section-3.1) **MUST** be `id.catalyst`.

When used as a Catalyst ID, where only catalyst IDs would be used, the scheme can be omitted.

### `authority`

The [authority](https://datatracker.ietf.org/doc/html/rfc3986#section-3.2) references the blockchain or network
the key was registered within.

It is perfectly valid for an ID Uri to reference a different network than the place where the ID or Key is used.
For example, a `cardano` ID can be used to post documents to `IPFS`.
Its purpose is to define WHERE the key was registered, and nothing more.

The Authority will consist of a `host` and optional `userinfo`.

#### `authority` - `host`

The [host](https://datatracker.ietf.org/doc/html/rfc3986#section-3.2.2)
refers to the network type where the RBAC registration was made.
It **IS NOT** resolvable with **DNS**, and **IS NOT** a public host name.
It is used as a decentralized network identifier.
The consumer of the `KID` must be able to resolve these host names.

The hostname may have one or more subdomains which could specify side-chains of a particular network,
or test networks.

##### List of defined hosts

| `host` | Description |
| --- | --- |
| `cardano` | Cardano Blockchain |
| `preprod.cardano` | Preprod Cardano Blockchain test network |
| `preview.cardano` | Preview Cardano Blockchain test network |
| `midnight` | Midnight Blockchain |
| `ethereum` | Ethereum Blockchain |
| `cosmos` | Cosmos Blockchain |

This list is indicative of the host names that can be used, any hostname is valid provided it is
capable of storing catalyst RBAC registration keychains.

#### `authority` - `userinfo`

The [userinfo] is used to hold a user defined readable name that can be attached to the keychain.
It may contain an optional `nonce` which is separated from the user's name by a `:` and replaces a
traditional password used for HTTP basic authentication.

Because the name is not unique, and is provided by the user, it is informational only.
A URI is identical, provided the hostname and path are the same, the [userinfo] does not play
a part in validating or finding the catalyst keychain being referenced.

The `nonce` part contained in the `password` component of the username *MUST* be an integer,
and it is the number of seconds since 1970 UTC, when the Catalyst ID URI was generated.

Applications which use the `nonce` will define its use, anything that does not use the `nonce` will ignore it.

##### Example `userinfo` with a `hostname`

* `anne@cardano` - username `anne` no nonce.
* `blake:1737101079@midnight` - username `blake` with nonce 1737101079.
* `:173710179#ethereum` - no username with nonce 173710179.

### `path`

The [path](https://datatracker.ietf.org/doc/html/rfc3986#section-3.3) defines the actual key within the registration.
Keys are defined relative to the very first Role0 Key registered in any RBAC registration.

The overall `path` specification is: `<initial role0 key>/<role>/<rotation>#encrypt`

* `<initial role 0 key>` - This is the very first role 0 key used to post the registration to the network.
  * It is the [Base64 URL] encoded binary data of the role 0 public key.
  * This does not change, even if the Initial Role 0 key is revoked.
  * This allows for an unambiguous identifier for the RBAC keychain.
  * It is not necessarily the key being identified.
  * An example Role 0 Key is `FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE`
* `<role>` - *Optional* This is the Role number being used.
  * It is a positive number, starting at 0, and no greater than 65535.
  * If it is not defined, then its default value is 0.
  * If it is not defined, there can be no `<rotation>` part of the path following.
* `<rotation>` - *Optional* This is the rotation of the defined role key being identified.
  * It starts at 0 for the first published key for the role, and increments by one for each subsequent published rotation.
  * This number refers to the published sequence of keys for the role in the RBAC registration keychain,
  not the index used in the key derivation.
  * It is positive and no greater than 65535.
  * If not present, it defaults to 0.
* `#encrypt` - [Fragment](https://datatracker.ietf.org/doc/html/rfc3986#section-3.5)
  disambiguates Encryption Public Keys from signing public keys.
  * Roles can have 1 active public signing key, and 1 active public encryption key.
  * By default, the URL is referencing the signing public key.
  * If a public encryption key is being identified, then the fragment `#encrypt` is appended to the [Universal Resource Identifier].

## Reference Implementation

The first implementation will be Catalyst Voices.

## Test Vectors

* `id.catalyst://cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE`
  * A Signing key registered on the Cardano Main network.
  * Role 0 - Rotation 0.
  * `username` - undefined.
  * `nonce` - undefined.
  * In this example, it is identical to `FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/0/0` or
  `FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/0`.
* `id.catalyst://cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/0`
  * A Signing key registered on the Cardano Main network.
  * Role 0 - Rotation 0.
  * `username` - undefined.
  * `nonce` - undefined.
  * In this example, it is identical to `FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/0/0` or
  `FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE`.
* `id.catalyst://gary@cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/0/0`
  * A Signing key registered on the Cardano Main network.
  * Role 0 - Rotation 0.
  * `username` - `gary`.
  * `nonce` - undefined.
  * In this example, it is identical to `FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE` or
  `FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/0`.
* `id.catalyst://faith@preprod@cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/7/3`
  * A Signing key registered on the Cardano pre-production network.
  * Role 7 - Rotation 3.
  * `username` - `faith`
  * `nonce` - undefined.
  * The Key for Role 7, and its third published rotation
  (i.e., the fourth key published, the first is the initial key, plus 3 rotations following it).
* `id.catalyst://faith:173710179@preprod@cardano/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/2/0#encrypt`
  * A Public Encryption key registered on the Cardano pre-production network.
  * Role 2 - Rotation 0.
  * `username` - `faith`
  * `nonce` - 173710179.
  * The initially published Public Encryption Key for Role 2.
* `kid.catalyst-rbac://:173710179@midnight/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/0/1`
  * A Signing key registered on the Midnight Blockchain Main network
  * Role 0 - Rotation 1.
  * `username` - undefined.
  * `nonce` - 173710179.
  * In this example, it is NOT the same as the `FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE`,
  as it identifies the first rotation after `FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE`.
* `kid.catalyst-rbac://midnight/FftxFnOrj2qmTuB2oZG2v0YEWJfKvQ9Gg8AgNAhDsKE/2/1#encrypt`
  * A public encryption key registered on the Midnight Blockchain Main network.
  * Role 2 - Rotation 1.
  * `username` - undefined.
  * `nonce` - 173710179.

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
[userinfo]: (https://datatracker.ietf.org/doc/html/rfc3986#section-3.2.1)
