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
* The `document_id` and `document+ver` must be within the time of allowed casting
    of ballots.  Any document_id of document_ver outside this time are invalid and will
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

IF [`ver`](../metadata.md#ver) does not == [`id`](../metadata.md#id) then a document with
[`id`](../metadata.md#id) and [`ver`](../metadata.md#ver) being equal *MUST* exist.

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

The document version must always be >= the document ID.

### [`ref`](../metadata.md#ref)

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Reference](../metadata.md#document-reference) |
| Multiple References | True |
| Valid References | [Rep Nomination](rep_nomination.md) |
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

The Payload is a [JSON][RFC8259] Document, and must conform to this schema.

It consists of an array which defines the weights to be applied to the chosen delegations.

Each valid delegate gets the matching weight from this array.
The total voting power is split proportionally based on these weights over the
valid drep nominations.
[CBOR][RFC8949] Payload Documentation

## Signers

The following User roles may sign documents of this type:

* Registered

Only the original author can update and sign a new version of documents.

## Copyright

| Copyright | :copyright: 2024-2025 IOG Singapore, All Rights Reserved |
| --- | --- |
| License | This document is licensed under [CC-BY-4.0] |
| Created | 2024-12-27 |
| Modified | 2025-10-24 |
| Authors | Alex Pozhylenkov <alex.pozhylenkov@iohk.io> |
| | Nathan Bogale <nathan.bogale@iohk.io> |
| | Neil McAuliffe <neil.mcauliffe@iohk.io> |
| | Steven Johnson <steven.johnson@iohk.io> |

### Changelog

#### 0.01 (2025-06-19)

* First Published Version

#### 0.1.2 (2025-09-04)

* Allow Multi Delegation

[CBOR-TAG-42]: https://github.com/ipld/cid-cbor/
[RFC9052-HeaderParameters]: https://www.rfc-editor.org/rfc/rfc8152#section-3.1
[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
[IPFS-CID]: https://docs.ipfs.tech/concepts/content-addressing/#what-is-a-cid
[RFC9562-V7]: https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-7
[RFC8949]: https://www.rfc-editor.org/rfc/rfc8949.html
[RFC8259]: https://www.rfc-editor.org/rfc/rfc8259.html
