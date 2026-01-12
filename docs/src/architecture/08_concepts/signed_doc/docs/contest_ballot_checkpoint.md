# Contest Ballot Checkpoint

## Description

Periodically, as ballots are collected, a summary of all newly collected ballots is
published in a [Contest Ballot Checkpoint](contest_ballot_checkpoint.md) document.
Each checkpoint accumulates state over time,
committing to the current set of accepted ballots via an SMT root and entry
count,
and optionally listing any ballots rejected in the same interval.

Checkpoint documents are chained together.
The final document in the sequence is indicated by the [`chain`](../metadata.md#chain) metadata.

Typically each [Contest Ballot Checkpoint](contest_ballot_checkpoint.md) is made immutable by referencing it on
the blockchain most applicable to the Contest.

Different blockchains will have different mechanisms for referencing checkpoint
documents.
For example, Cardano can encode a `document_ref` in on‑chain metadata,
signed by the ballot‑box (bulletin board) operator.

The blockchain record should be as close in time as practically possible to the
creation of the [Contest Ballot Checkpoint](contest_ballot_checkpoint.md) document to provide a reliable anchor for
proofs of inclusion and auditability.

<!-- markdownlint-disable max-one-sentence-per-line -->

```graphviz dot contest_ballot_checkpoint.dot.svg

{{ include_file('./../diagrams/contest_ballot_checkpoint.dot', indent=4) }}
```

<!-- markdownlint-enable max-one-sentence-per-line -->

### Validation

* [`parameters`](../metadata.md#parameters) metadata MUST reference the Contest this checkpoint pertains to.
* [`ref`](../metadata.md#ref) metadata MUST reference the accepted Contest Ballots collected in the preceding
  interval by the bulletin board.
  Entries MUST be sorted by ascending `document_id`:`document_ver`,
  regardless of the arrival time at the bulletin board.
* Ballot boxes MUST reject ballots whose `document_id`:`document_ver` fall outside the
  contest’s allowed time window,
  or that are not close in time to when the ballot box received the ballot.
* When present, `rejections` MUST only contain recognized reasons and valid
  `document_ref` values of Contest Ballot documents;
  rejected ballots MUST NOT appear in [`ref`](../metadata.md#ref) for the same interval.
* `smt-root` MUST be the Blake3 root hash of the canonical SMT containing all accepted
  ballots up to and including this checkpoint;
* `smt-entries` MUST equal the total count of leaves in that SMT.
* [`chain`](../metadata.md#chain) MUST be intact and consistent:
  the previous checkpoint referenced by [`chain`](../metadata.md#chain)
  MUST exist, match type, id, and parameters, and have a lower [`ver`](../metadata.md#ver) and height exactly
  one less than this checkpoint.

### Business Logic

#### Front End

* Not produced by the Front End.
* May be read to verify that a proof of inclusion validates against the published
  `smt-root` and `smt-entries`.

#### Back End

* Validate that all referenced ballots exist and are valid for the contest.
* Ensure the document is signed by an authoritative bulletin‑board operator.
* Ensure all referenced ballots are for the same contest as [`parameters`](../metadata.md#parameters).
* Compute and verify `smt-root` and `smt-entries` against the current SMT state.
* If present, validate `rejections` reasons and that rejected `document_ref`s are
  Contest Ballot documents.
* Ensure the chain is intact and consistent with the previous checkpoint.
* Ensure no previous checkpoint already chains to the same target (no forks within a
  single authoritative sequence).

## [COSE Header Parameters][RFC9052-HeaderParameters]

* [content type](../spec.md#content-type) = `application/cbor`
* [content-encoding](../spec.md#content-encoding) = `[br]`

## Metadata

### [`type`](../metadata.md#type)

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Type](../metadata.md#document-type) |
| Type | 58608925-bda3-47df-b39a-ae0d0a1dd6ed |
<!-- markdownlint-enable MD033 -->
The document TYPE.

#### [`type`](../metadata.md#type) Validation

**MUST** be a known document type.

### [`id`](../metadata.md#id)

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Id](../metadata.md#document-id) |
<!-- markdownlint-enable MD033 -->
Document ID, created the first time the document is created.
This must be a properly created [UUIDv7][RFC9562-V7] which contains the
timestamp of when the document was created.

#### [`id`](../metadata.md#id) Validation

The document ID validation is performed based on timestamp thresholds:

* If `future_threshold` is configured,
the document [`id`](../metadata.md#id) cannot be too far in the future from the
current time.
* If `past_threshold` is configured, the document [`id`](../metadata.md#id) cannot be too far in the past from the
current time.

### [`ver`](../metadata.md#ver)

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Ver](../metadata.md#document-ver) |
<!-- markdownlint-enable MD033 -->
The unique version of the document.
The first version of the document must set [`ver`](../metadata.md#ver) == [`id`](../metadata.md#id)

[`ver`](../metadata.md#ver) represents new versions of the same document as it changes over time.

#### [`ver`](../metadata.md#ver) Validation

1. The document version must always be >= the document ID.
2. IF [`ver`](../metadata.md#ver) does not == [`id`](../metadata.md#id)
  then a document with [`id`](../metadata.md#id) and [`ver`](../metadata.md#ver) being equal *MUST* exist.
3. When a document with the same [`id`](../metadata.md#id) already exists,
  the new document's [`ver`](../metadata.md#ver) must be greater than
  the latest known submitted version for that [`id`](../metadata.md#id).
4. When a document with the same [`id`](../metadata.md#id) already exists,
  the new document's [`type`](../metadata.md#type) must be the same as
  the latest known submitted document's [`type`](../metadata.md#type) for that [`id`](../metadata.md#id).

### [`ref`](../metadata.md#ref)

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Reference](../metadata.md#document-reference) |
| Multiple References | True |
| Valid References | [Contest Ballot](contest_ballot.md) |
<!-- markdownlint-enable MD033 -->
Reference to a Linked Document or Documents.
This is the primary hierarchical reference to a related document.

If a reference is defined as required, there must be at least 1 reference specified.
Some documents allow multiple references, and they are documented as required.

The document reference serves two purposes:

1. It ensures that the document referenced by an ID/Version is not substituted.
    In other words, that the document intended to be referenced, is actually referenced.
2. It Allows the document to be unambiguously located in decentralized storage systems.

There can be any number of Document Locations in any reference.
The currently defined locations are:

* `cid` : A [CBOR Encoded IPLD Content Identifier][CBOR-TAG-42] ( AKA an [IPFS CID][IPFS-CID] ).
* Others may be added when further storage mechanisms are defined.

The document location does not guarantee that the document is actually stored.
It only defines that if it were stored, this is the identifier
that is required to retrieve it.
Therefore it is required that Document References
are unique and reproducible, given a documents contents.

#### [`ref`](../metadata.md#ref) Validation

The following must be true for a valid reference:

* The Referenced Document **MUST** Exist
* Every value in the `document_locator` must consistently reference the exact same document.
* The `document_id` and `document_ver` **MUST** match the values in the referenced document.
* In the event there are **MULTIPLE** [`ref`](../metadata.md#ref) listed, they **MUST** be sorted.

Sorting for each element of [`ref`](../metadata.md#ref) follows the same sort order as specified for Map Keys,
as defined by [CBOR Deterministic Encoding][CBOR-LFD-ENCODING] (4.3.2 Length-First Map Key Ordering).

### [`parameters`](../metadata.md#parameters)

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Reference](../metadata.md#document-reference) |
| Valid References | [Contest Parameters](contest_parameters.md) |
| Linked Reference Metadata | [`ref`](#ref) |
<!-- markdownlint-enable MD033 -->
A reference to the Parameters Document this document lies under.

#### [`parameters`](../metadata.md#parameters) Validation

In addition to the validation performed for [Document Reference](../metadata.md#document-reference) type fields:

* Any linked referenced document that includes a [`parameters`](../metadata.md#parameters) metadata must match the
[`parameters`](../metadata.md#parameters) of the referencing document,
or a parent of those [`parameters`](../metadata.md#parameters).

For example, a linked reference to [Contest Parameters](contest_parameters.md) is transitively a reference to
the Parameters document it references, and each parameters document they reference
until the `Brand` parameters document is reached.

The use case here is for Templates.
The profile template, or proposal templates could be defined at any of these
levels, and as long as they all refer to the same chain of parameters in the
hierarchy they are all valid.

* The Document referenced by [`ref`](../metadata.md#ref)
    * MUST contain [`parameters`](../metadata.md#parameters) metadata; AND
    * MUST match the referencing documents [`parameters`](../metadata.md#parameters) value.

### [`chain`](../metadata.md#chain)

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Chain Link](../metadata.md#chain-link) |
<!-- markdownlint-enable MD033 -->
An immutable link to the previous document in a chained sequence of documents.
Because ID/Ver only defines values for the current document, and is not intended
by itself to prevent insertion of documents in a sequence, the [`chain`](../metadata.md#chain)
metadata allows for the latest document to directly point to its previous iteration.

It also aids in discoverability, where the latest document may be pinned but prior
documents can be discovered automatically by following the chain.

#### [`chain`](../metadata.md#chain) Validation

Chained Documents do not support collaborators.
Any document which is attempted to be published in the sequence
which is *NOT* published by the author of the first document in the
sequence is fraudulent, and to be discarded.

In addition, the chained document *MUST*:

* Not have `collaborators`;
* Have the same [`id`](../metadata.md#id) as the document being chained to;
* Have a [`ver`](../metadata.md#ver) that is greater than the [`ver`](../metadata.md#ver) being chained to;
* Have the same [`type`](../metadata.md#type) as the chained document;
* Have [`parameters`](../metadata.md#parameters) match;
* Have not be chaining to a document already chained to by another document;
* Have its absolute `height` exactly one more than the `height` of the document being chained to.

IF any of these validations fail, then the entire sequence of documents is INVALID.
Not just the current document.

## Payload

The Payload is a [CBOR][RFC8949] document that MUST conform to the
`contest-ballot-checkpoint` [CDDL][RFC8610] schema.

Contents

* `stage` (required)
    * Processing stage represented by this checkpoint.
    * One of: `"bulletin-board" | "tally" | "audit"`.

* `smt-root` (required)
    * Blake3 256‑bit digest of the root of the Sparse Merkle Tree (SMT)
      containing all accepted ballot `document_ref`s up to and including
      this checkpoint.

* `smt-entries` (required)
    * The total number of documents (leaves) in the SMT at this checkpoint.

* `rejections` (optional)
    * Map of `rejection-reason => [ document_ref, ... ]` listing ballots
      rejected during this checkpoint interval.
    * Reasons are limited to: `"already-voted"`, `"obsolete-vote"`.

* `encrypted-tally` (optional)
    * Placeholder map of `document_ref => encrypted-tally-proposal-result`.
    * May appear at later stages to commit to encrypted tally snapshots.

* `tally` (optional)
    * Placeholder map of `document_ref => tally-proposal-result` for clear tally
      snapshots.

* `drep-encryption-key` (optional)
    * Placeholder for a DRep encryption key to allow decryption where required
      for audit or published results.

Notes

* The document [`ref`](../metadata.md#ref) metadata lists the accepted Contest Ballots collected during
  the interval covered by this checkpoint;
  rejected ballots are listed under `rejections` and are not included in [`ref`](../metadata.md#ref) for that interval.
* The SMT is cumulative across the chain; each checkpoint’s `smt-root` and
  `smt-entries` commit to all accepted ballots up to that point.

### Schema
<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Payload [CDDL][RFC8610] Schema"

    * [contest_ballot_checkpoint.cddl](../cddl/contest_ballot_checkpoint.cddl)

    ``` cddl
    {{ include_file('./../cddl/contest_ballot_checkpoint.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

#### Sub-schemas

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: stage"

    * [stage.cddl](../cddl/stage.cddl)

    ``` cddl
    {{ include_file('./../cddl/stage.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: smt-root"

    * [smt_root.cddl](../cddl/smt_root.cddl)

    ``` cddl
    {{ include_file('./../cddl/smt_root.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: blake3"

    * [blake3.cddl](../cddl/blake3.cddl)

    ``` cddl
    {{ include_file('./../cddl/blake3.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: smt-entries"

    * [smt_entries.cddl](../cddl/smt_entries.cddl)

    ``` cddl
    {{ include_file('./../cddl/smt_entries.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: rejections"

    * [rejections.cddl](../cddl/rejections.cddl)

    ``` cddl
    {{ include_file('./../cddl/rejections.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: rejection-reason"

    * [rejection_reason.cddl](../cddl/rejection_reason.cddl)

    ``` cddl
    {{ include_file('./../cddl/rejection_reason.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: document_ref"

    * [document_ref.cddl](../cddl/document_ref.cddl)

    ``` cddl
    {{ include_file('./../cddl/document_ref.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: document_id"

    * [document_id.cddl](../cddl/document_id.cddl)

    ``` cddl
    {{ include_file('./../cddl/document_id.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: uuid_v7"

    * [uuid_v7.cddl](../cddl/uuid_v7.cddl)

    ``` cddl
    {{ include_file('./../cddl/uuid_v7.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: document_ver"

    * [document_ver.cddl](../cddl/document_ver.cddl)

    ``` cddl
    {{ include_file('./../cddl/document_ver.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: document_locator"

    * [document_locator.cddl](../cddl/document_locator.cddl)

    ``` cddl
    {{ include_file('./../cddl/document_locator.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: cid"

    * [cid.cddl](../cddl/cid.cddl)

    ``` cddl
    {{ include_file('./../cddl/cid.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: cbor-cid"

    * [cbor_cid.cddl](../cddl/cbor_cid.cddl)

    ``` cddl
    {{ include_file('./../cddl/cbor_cid.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: encrypted-tally"

    * [encrypted_tally.cddl](../cddl/encrypted_tally.cddl)

    ``` cddl
    {{ include_file('./../cddl/encrypted_tally.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: encrypted-tally-proposal-result"

    * [encrypted_tally_proposal_result.cddl](../cddl/encrypted_tally_proposal_result.cddl)

    ``` cddl
    {{ include_file('./../cddl/encrypted_tally_proposal_result.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: tally"

    * [tally.cddl](../cddl/tally.cddl)

    ``` cddl
    {{ include_file('./../cddl/tally.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: tally-proposal-result"

    * [tally_proposal_result.cddl](../cddl/tally_proposal_result.cddl)

    ``` cddl
    {{ include_file('./../cddl/tally_proposal_result.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: clear-choice"

    * [clear_choice.cddl](../cddl/clear_choice.cddl)

    ``` cddl
    {{ include_file('./../cddl/clear_choice.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: voting-power"

    * [voting_power.cddl](../cddl/voting_power.cddl)

    ``` cddl
    {{ include_file('./../cddl/voting_power.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: drep-encryption-key"

    * [drep_encryption_key.cddl](../cddl/drep_encryption_key.cddl)

    ``` cddl
    {{ include_file('./../cddl/drep_encryption_key.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

## Signers

The following Admin roles may sign documents of this type:

* Bulletin Board Operator

Only the original author can update and sign a new version of documents.

## Copyright

| Copyright | :copyright: 2024-2026 IOG Singapore, All Rights Reserved |
| --- | --- |
| License | This document is licensed under [CC-BY-4.0] |
| Created | 2024-12-27 |
| Modified | 2026-01-09 |
| Authors | Alex Pozhylenkov <alex.pozhylenkov@iohk.io> |
| | Nathan Bogale <nathan.bogale@iohk.io> |
| | Neil McAuliffe <neil.mcauliffe@iohk.io> |
| | Steven Johnson <steven.johnson@iohk.io> |

### Changelog

#### 0.1.5 (2025-11-03)

* Add Voting Ballots and Ballot Checkpoint Documents

[CBOR-TAG-42]: https://github.com/ipld/cid-cbor/
[CBOR-LFD-ENCODING]: https://www.rfc-editor.org/rfc/rfc8949.html#section-4.2.3
[RFC9052-HeaderParameters]: https://www.rfc-editor.org/rfc/rfc8152#section-3.1
[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
[IPFS-CID]: https://docs.ipfs.tech/concepts/content-addressing/#what-is-a-cid
[RFC9562-V7]: https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-7
[RFC8949]: https://www.rfc-editor.org/rfc/rfc8949.html
[RFC8610]: https://www.rfc-editor.org/rfc/rfc8610
