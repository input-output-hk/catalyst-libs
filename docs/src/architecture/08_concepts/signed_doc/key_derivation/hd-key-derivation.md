---
Title: Catalyst HD Key Derivation for Off Chain Keys
Authors:
    - Steven Johnson <steven.johnson@iohk.io>
Created: 2024-11-29
License: [CC-BY-4.0]
---

## Abstract

Project Catalyst uses off chain keys, as a proxy for on-chain keys.
These keys need to be derived similar to the keys controlled by a wallet.
This document defines the Derivation path.

## Motivation: why is this CIP necessary?

A user will need a number of self generated and controlled signature and other keys.
They will need to be able to recover them from a known seed phrase, and also to roll them over.

This allows users to replace keys, and have them fully recoverable.
Which they may have to do if:

* Their keys are lost, and the account has to be recovered, or moved to a different device.
* Their keys are compromised (or suspected to be compromised), and they have to be replaced.

The keys are not controlled by a Blockchain wallet.
They are agnostic of any blockchain.
So, Project Catalyst must implement similar mechanisms as the wallets to safely derive keys for its use.

## Specification

For reference, see [CIP-1852].
This document is a modified implementation of this specification.

The basic structure of the Key Derivation path shall be:

```text
m / purpose' / type' / usage' / role / index
```

### `purpose'`

Defines the purpose of the key, a distinct value from the one chosen for cardano is used to
prevent collision with keys derived by wallets if the same seed phrase were to be used.
Cardano uses a notable year that aligns with Cardano ecosystem philosophy,
we maintain that practice but choose alternative [historical dates].

* Value: `508`
* Name in [CIP-1852]: `purpose'`
* Hardened: YES
* Represents: Taken from year 508 BCE, the first known instance of democracy in human history.
    *"The Athenian Revolution, a revolt that overthrew the aristocratic oligarchy and
    established a participatory democracy in Athens"*.

### `type'`

Defines the type of the key, a distinct value from the one chosen for cardano is used to
prevent collision with keys derived by wallets if the same seed phrase were to be used.
Cardano uses a notable year that aligns with Cardano ecosystem philosophy,
we maintain that practice but choose alternative [historical dates].

* Value: `139`
* Name in [CIP-1852]: `coin_type'`
* Hardened: YES
* Represents: Taken from the year 139 BCE, the first known instance of secret voting.
    *"A secret ballot is instituted for Roman citizens, who mark their vote on a tablet and
    place it in an urn."*

### `usage'`

Defines how the derived key will be used.
This occupies the same position as `account'` in [CIP-1852].

* Value: positive integer (0-n)
* Name in [CIP-1852]: `account'`
* Hardened: YES

| `usage'` | Name |
| 0 | [ED25519 Document Signing Key](./ed25519-document-signing-key-derivation.md) |
| 1 | Used to derive a Root Symmetric encryption key used for encrypting data within a document. |
| 2+ | Currently undefined. |

### `role`

The role in the derivation maps 1:1 with the role number in the RBAC registration the key will be used for.

* Value: positive integer (0-n)
* Name in [CIP-1852]: `role`
* Hardened: NO

### `index`

The sequentially derived key in a sequence, starting at 0.
Each new key for the same role just increments `index`.
This is mapped 1:1 to the `rotation` field in a Catalyst ID.

* Value: positive integer (0-n)
* Name in [CIP-1852]: `index`
* Hardened: NO

## Reference Implementation

The first implementation will be for
[ED25519 Document Signing Key](./ed25519-document-signing-key-derivation.md)
in Catalyst Voices.

*TODO: Generate a set of test vectors which conform to this specification.*

## Rationale: how does this CIP achieve its goals?

By leveraging known working Key Derivation techniques and simply modifying the path we inherit the properties of those methods.

## Path to Active

### Acceptance Criteria

Working Implementation before Fund 14.

### Implementation Plan

Fund 14 project catalyst will deploy this scheme for Key derivation.>

## Copyright

This document is licensed under [CC-BY-4.0]

[historical dates]: https://www.oxfordreference.com/display/10.1093/acref/9780191737152.timeline.0001
[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
