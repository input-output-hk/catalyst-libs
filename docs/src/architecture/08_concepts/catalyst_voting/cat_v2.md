# Catalyst V2

---

Title: Catalyst V2 Voting Transaction

Status: Proposed

Authors:
    - Alex Pozhylenkov <alex.pozhylenkov@iohk.io>

Created: 2024-10-24

---

## Abstract

This document describes a Catalyst V2 vote transaction structure.

## Motivation

## Specification

It is a Catalyst v2 voting transaction
defined on top the ["Generalized Vote Transaction"](./gen_vote_tx.md#specification) structure.

Following that spec need to define a format of `choice`, `proof` and `prop_id`.

<!-- markdownlint-disable max-one-sentence-per-line -->
!!! note

    - If `choice` is *public* one, `proof` **must** be `null`.
    - If `choice` is *private* one, `proof` **must** be **not** `null`.
<!-- markdownlint-disable max-one-sentence-per-line -->

<!-- markdownlint-disable max-one-sentence-per-line code-block-style -->
??? note "vote transaction v2 definition: `vote_tx_v2.cddl`"

    ```CDDL
    {{ include_file('src/architecture/08_concepts/catalyst_voting/cddl/vote_tx_v2.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line code-block-style -->

### Vote generation

To generate a cryptographically secured `private_choice` and `zk_proof` parts you can follow this [spec](./crypto.md#vote).
Important to note,
that as part of [*initial setup*](./crypto.md#initial-setup) of the voting procedure,
the following properties are used:

1. Each proposal,
   defined by the `vote_plan_id` and `proposal_index`, defines a number of possible options.
2. [ristretto255] as a backend cryptographic group.
3. A commitment key $ck$ defined as a [BLAKE2b-512] hash of the `vote_plan_id` bytes.

## Rationale

## Path to Active

### Acceptance Criteria
<!-- Describes what are the acceptance criteria whereby a proposal becomes 'Active' -->

### Implementation Plan
<!-- A plan to meet those criteria or `N/A` if an implementation plan is not applicable. -->

<!-- OPTIONAL SECTIONS: see CIP-0001 > Document > Structure table -->

[BLAKE2b-512]: https://www.blake2.net/blake2.pdf
[ristretto255]: https://ristretto.group
<!-- [COSE]: https://datatracker.ietf.org/doc/rfc9052/ -->
<!-- [CBOR]: https://datatracker.ietf.org/doc/rfc8949/ -->