# General Voting Transaction

---

Title: General Voting Transaction Structure

Status: Proposed

Authors:
    - Alex Pozhylenkov <alex.pozhylenkov@iohk.io>

Created: 2024-09-04

---

## Abstract

This document defines a generalized view of the "Catalyst" voting transaction.

## Motivation

Project "Catalyst" requires a structure to keep people vote's data in the secure and verifiable way.

## Specification

Generalized vote transaction is a [signed object],
so its fully follows the structure of the [signed object] specification.

* [`content type`](./../signed_object/index.md#content-type): `application/cbor`.
  [Signed object content](./../signed_object/index.md#signed-object-content) must be a [CBOR] encoded.
  ```CDDL
  3 => 50
  ```
* [`content encoding`](./../signed_object/index.md#content-encoding-optional): is missing

### Content format

The generalized vote transaction [content format](./../signed_object/index.md#signed-object-content)

<!-- markdownlint-disable max-one-sentence-per-line code-block-style -->
??? note "vote transaction definition: `gen_vote_tx.cddl`"

    ```CDDL
    {{ include_file('src/architecture/08_concepts/catalyst_voting/cddl/gen_vote_tx.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line code-block-style -->

Vote:

* `choices` - a collection of voter choices for the proposal.
* `proof` - a voter proof, could be `null`.
* `prop-id` - a proposal id for which `choice` is made, could be `null`.
  For that case where for the `event` defined only **one** proposal,
  so it's redundant to provide an additional identifier for the proposal,
  so it could be placed `null`.

`voter-data` - an any additional voter's specific data.

## Rationale

## Path to Active

### Acceptance Criteria
<!-- Describes what are the acceptance criteria whereby a proposal becomes 'Active' -->

### Implementation Plan
<!-- A plan to meet those criteria or `N/A` if an implementation plan is not applicable. -->

<!-- OPTIONAL SECTIONS: see CIP-0001 > Document > Structure table -->

[signed object]: ./../signed_object/index.md
[BLAKE2b-256]: https://www.blake2.net/blake2.pdf
[COSE]: https://datatracker.ietf.org/doc/rfc9052/
[CBOR]: https://datatracker.ietf.org/doc/rfc8949/
