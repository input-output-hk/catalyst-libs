# General Voting Transaction

---

Title: General Voting Transaction Structure

Status: Proposed

Authors:
    - Alex Pozhylenkov <alex.pozhylenkov@iohk.io>

Created: 2024-09-04

---

## Abstract

This document defines a generilized view of the "Catalyst" voting transaction.

## Motivation

Project "Catalyst" requires a structure to keep people vote's data in the secure way, anonymous and verifiable way.

## Specification

<!-- markdownlint-disable max-one-sentence-per-line code-block-style -->
??? note "vote transaction definition: `gen_vote_tx.cddl`"

    ```CDDL
    {{ include_file('src/architecture/08_concepts/catalyst_voting/cddl/gen_vote_tx.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line code-block-style -->

`event` - a set of different identifiers which is uniquely define a particular voting event.

Vote:

* `choices` - a collection of voter choices for the proposal.
* `proof` - a voter proof, could be `null`.
* `prop-id` - a proposal id for which `choice` is made, could be `null`.
  For that case where for the `event` defined only **one** proposal,
  so it's redundant to provide an additional identifier for the proposal,
  so it could be placed `null`.

`voters-data` - an any additional voter's specific data.

### Transaction signing

[COSE] is used to define a transaction's signature structure.
[COSE] is a flexible security protocol that supports various types of security messages.
However, only `COSE Signed Data Object` or `COSE_Sign` type is used.

The following header must be included in the [COSE] signature.

`protected`:

* `content type`: `application/cbor`
  (this parameter is used to indicate the content type of the data in the payload or ciphertext fields).

Any other headers as `alg`, `kid` etc. could be specified of any kind and not defined by this spec.

#### Signature payload

As mentioned earlier, the content type of the [COSE] signature payload is `application/cbor`.
In particular it must be a [CBOR] encoded [BLAKE2b-256] hash bytes:

<!-- markdownlint-disable code-block-style -->
```CDDL
{{ include_file('src/architecture/08_concepts/catalyst_voting/cddl/get_vote_tx_cose_payload.cddl') }}
```
<!-- markdownlint-enable code-block-style -->

## Rationale

## Path to Active

### Acceptance Criteria
<!-- Describes what are the acceptance criteria whereby a proposal becomes 'Active' -->

### Implementation Plan
<!-- A plan to meet those criteria or `N/A` if an implementation plan is not applicable. -->

<!-- OPTIONAL SECTIONS: see CIP-0001 > Document > Structure table -->

[BLAKE2b-256]: https://www.blake2.net/blake2.pdf
[COSE]: https://datatracker.ietf.org/doc/rfc9052/
[CBOR]: https://datatracker.ietf.org/doc/rfc8949/
