# Contest Ballot

## Description

An individual Ballot cast in a Contest by a registered user.

Each ballot contains choices for all possible proposals eligible for the
contest.

Multiple contest ballots can be cast by the same registered user in a contest, but
only the latest (by its document_version) will be counted.

The reason the ballot is cast in a contest is because there may be multiple contests in
a campaign, and they may be attached to either the brand, campaign or category level.
Each level, (for example category) can in-theory have multiple contests.

Only eligible users can cast ballots in the respective contest.

<!-- markdownlint-disable max-one-sentence-per-line -->

```graphviz dot contest_ballot.dot.svg

{{ include_file('./../diagrams/contest_ballot.dot', indent=4) }}
```

<!-- markdownlint-enable max-one-sentence-per-line -->

### Validation

* The [`parameters`](../metadata.md#parameters) metadata *MUST* point to the Contest the ballot is being cast in.
* The 'ref' metadata fields within the ballot payload (not the headers) must point to
  ALL the proposals eligible to be chosen in the contest.

### Business Logic

#### Front End

* Always cast a ballot for all proposals in the contest.
* Any proposal not explicitely selected by a user must have the default selection applied.
  Typically, this would be `abstain`.
* The voter signs this document to confirm their ballot.
* Ballots can not be cast outside the time allowed for the casting of ballots.
* The `document_id` and `document_ver` must be within the time of allowed casting
  of ballots.
  Any `document_id` or `document_ver` outside this time are invalid and will
  not be counted.

#### Back End

* Verifies that the Contest is valid, and that the ballot is cast in the appropriate
  time frame, and has a valid `document_id` and `document_ver` in that range.
* Verify the payload lists all the eligible proposals which can be chosen in the contest.
* Verify the proofs in the payload are correct.

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
| Type | de1284b8-8533-4f7a-81cc-ff4bde5ef8d0 |
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
| Valid References | [Proposal](proposal.md) |
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

### [`revocations`](../metadata.md#revocations)

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | optional |
| Format | [Version Revocations](../metadata.md#version-revocations) |
<!-- markdownlint-enable MD033 -->
A document may include a list of any prior versions which are considered to be revoked.
Only the revocation list in the latest version of the document applies.
Revoked documents are flagged as no longer valid, and should not be displayed.
As a special case, if the revocations are set to `true` then all versions of the document
are revoked, including the latest document.

In this case, when the latest document is revoked, the payload may be `nil`.
Any older document that has [`revocations`](../metadata.md#revocations) set to `true` is always to be filtered
and its payload is to be assumed to be invalid.

This allows for an entire document and any/all published versions to be revoked.
A new version of the document that is published after this, may reinstate prior
document versions, by not listing them as revoked.
However, any document where revocations was set `true` can never be reinstated.

#### [`revocations`](../metadata.md#revocations) Validation

If the field is `true` the payload may be absent or invalid.
Such documents may never be submitted.

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

## Payload

The Payload is a [CBOR][RFC8949] document that must conform to the `contest-ballot-payload` [CDDL][RFC8610].

Contents

* `uint => choices`
    * The payload is a map keyed by a `uint` index to the array element from [`ref`](../metadata.md#ref) metadata field,
      which is a proposal `document_ref`.
    * Each identifies one specific proposal via `[document_id, document_ver, document_locator]`.
    * The value for each key is that voter’s `choices` for that proposal.
    * There is exactly one set of `choices` per referenced proposal (no duplicates).

* `choices`
    * Discriminated union of unencrypted or encrypted choices.

* `row-proof` (optional, inside encrypted choices)
    * Proves, without revealing contents, that the encrypted row encodes a unit vector with exactly one selection.

* `column-proof` (optional, top-level)
    * Placeholder for future column-level proofs across proposals.
    * Not defined at present; omit in implementations.

* `matrix-proof` (optional, top-level)
    * Placeholder for future matrix-wide proofs across all proposals and positions.
    * Not defined at present; omit in implementations.

* `voter-choice` (optional, top-level)
    * This is ONLY Not included when the vote is unencrypted.
    * Allows a voter to read back their ballot selections without decrypting the entire ballot.

Notes

* `document_locator` uses a [CBOR][RFC8949] Tag 42 `cid` to locate the referenced proposal in content-addressed storage.
  Implementations should constrain the CID to SHA2-256 multihash; the multicodec SHOULD be `cbor (0x51)` to
  reflect an unwrapped COSE_Sign [CBOR][RFC8949] block.
* The application defines the permissible range and semantics of `clear-choice` integers.
* All [CBOR][RFC8949] must use core-deterministic encoding so that content addressing remains stable.

### Schema
<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Payload [CDDL][RFC8610] Schema"

    * [contest_ballot_payload.cddl](../cddl/contest_ballot_payload.cddl)

    ``` cddl
    {{ include_file('./../cddl/contest_ballot_payload.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

#### Sub-schemas

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: choices"

    * [choices.cddl](../cddl/choices.cddl)

    ``` cddl
    {{ include_file('./../cddl/choices.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: clear-choices"

    * [clear_choices.cddl](../cddl/clear_choices.cddl)

    ``` cddl
    {{ include_file('./../cddl/clear_choices.cddl', indent=4) }}
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
??? note "Required Definition: elgamal-ristretto255-encrypted-choices"

    * [elgamal_ristretto255_encrypted_choices.cddl](../cddl/elgamal_ristretto255_encrypted_choices.cddl)

    ``` cddl
    {{ include_file('./../cddl/elgamal_ristretto255_encrypted_choices.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: elgamal-ristretto255-encrypted-choice"

    * [elgamal_ristretto255_encrypted_choice.cddl](../cddl/elgamal_ristretto255_encrypted_choice.cddl)

    ``` cddl
    {{ include_file('./../cddl/elgamal_ristretto255_encrypted_choice.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: elgamal-ristretto255-group-element"

    * [elgamal_ristretto255_group_element.cddl](../cddl/elgamal_ristretto255_group_element.cddl)

    ``` cddl
    {{ include_file('./../cddl/elgamal_ristretto255_group_element.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: row-proof"

    * [row_proof.cddl](../cddl/row_proof.cddl)

    ``` cddl
    {{ include_file('./../cddl/row_proof.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: zkproof-elgamal-ristretto255-unit-vector-with-single-selection"

    * [zkproof_elgamal_ristretto255_unit_vector_with_single_selection.cddl](../cddl/zkproof_elgamal_ristretto255_unit_vector_with_single_selection.cddl)

    ``` cddl
    {{ include_file('./../cddl/zkproof_elgamal_ristretto255_unit_vector_with_single_selection.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: zkproof-elgamal-ristretto255-unit-vector-with-single-selection-item"

    * [zkproof_elgamal_ristretto255_unit_vector_with_single_selection_item.cddl](../cddl/zkproof_elgamal_ristretto255_unit_vector_with_single_selection_item.cddl)

    ``` cddl
    {{ include_file('./../cddl/zkproof_elgamal_ristretto255_unit_vector_with_single_selection_item.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: zkproof-elgamal-announcement"

    * [zkproof_elgamal_announcement.cddl](../cddl/zkproof_elgamal_announcement.cddl)

    ``` cddl
    {{ include_file('./../cddl/zkproof_elgamal_announcement.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: zkproof-elgamal-group-element"

    * [zkproof_elgamal_group_element.cddl](../cddl/zkproof_elgamal_group_element.cddl)

    ``` cddl
    {{ include_file('./../cddl/zkproof_elgamal_group_element.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: zkproof-ed25519-r-response"

    * [zkproof_ed25519_r_response.cddl](../cddl/zkproof_ed25519_r_response.cddl)

    ``` cddl
    {{ include_file('./../cddl/zkproof_ed25519_r_response.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: zkproof-ed25519-scalar"

    * [zkproof_ed25519_scalar.cddl](../cddl/zkproof_ed25519_scalar.cddl)

    ``` cddl
    {{ include_file('./../cddl/zkproof_ed25519_scalar.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: column-proof"

    * [column_proof.cddl](../cddl/column_proof.cddl)

    ``` cddl
    {{ include_file('./../cddl/column_proof.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: matrix-proof"

    * [matrix_proof.cddl](../cddl/matrix_proof.cddl)

    ``` cddl
    {{ include_file('./../cddl/matrix_proof.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: voter-choice"

    * [voter_choice.cddl](../cddl/voter_choice.cddl)

    ``` cddl
    {{ include_file('./../cddl/voter_choice.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: aes-ctr-encrypted-choices"

    * [aes_ctr_encrypted_choices.cddl](../cddl/aes_ctr_encrypted_choices.cddl)

    ``` cddl
    {{ include_file('./../cddl/aes_ctr_encrypted_choices.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "Required Definition: aes-ctr-encrypted-block"

    * [aes_ctr_encrypted_block.cddl](../cddl/aes_ctr_encrypted_block.cddl)

    ``` cddl
    {{ include_file('./../cddl/aes_ctr_encrypted_block.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

## Signers

The following User roles may sign documents of this type:

* Registered

Only the original author can update and sign a new version of documents.

## Copyright

| Copyright | :copyright: 2024-2026 IOG Singapore, All Rights Reserved |
| --- | --- |
| License | This document is licensed under [CC-BY-4.0] |
| Created | 2024-12-27 |
| Modified | 2026-01-13 |
| Authors | Alex Pozhylenkov <alex.pozhylenkov@iohk.io> |
| | Nathan Bogale <nathan.bogale@iohk.io> |
| | Neil McAuliffe <neil.mcauliffe@iohk.io> |
| | Steven Johnson <steven.johnson@iohk.io> |

### Changelog

#### 0.1.5 (2025-11-03)

* Add Voting Ballots and Ballot Checkpoint Documents

#### 0.2.1 (2025-12-02)

* Added missing [`ref`](../metadata.md#ref) metadata field definition.
* Improved `payload` [cddl][RFC8610] definition, replaced `document_ref` to the `uint` as a map keys to the `choices`.

[CBOR-TAG-42]: https://github.com/ipld/cid-cbor/
[CBOR-LFD-ENCODING]: https://www.rfc-editor.org/rfc/rfc8949.html#section-4.2.3
[RFC9052-HeaderParameters]: https://www.rfc-editor.org/rfc/rfc8152#section-3.1
[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
[IPFS-CID]: https://docs.ipfs.tech/concepts/content-addressing/#what-is-a-cid
[RFC9562-V7]: https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-7
[RFC8949]: https://www.rfc-editor.org/rfc/rfc8949.html
[RFC8610]: https://www.rfc-editor.org/rfc/rfc8610
