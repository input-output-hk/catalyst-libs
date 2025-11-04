# Contest Ballot Checkpoint

## Description

Periodically as ballots are collected, a summary of all newly collected ballots will be
published in a [Contest Ballot Checkpoint](contest_ballot_checkpoint.md) document.
This document forms part of the bulletin boards complete Contest Ballot Checkpoint.

These documents are chained to each other, and the final document is specified as final
in the [`chain`](../metadata.md#chain) metadata.

Typically each [Contest Ballot Checkpoint](contest_ballot_checkpoint.md) document is made immutable by referencing it on
the blockchain most applicable to the Contest.

Different blockchains will have different mechanisms for referencing the individual
[Contest Ballot Checkpoint](contest_ballot_checkpoint.md) documents.

For example, Cardano will encode a `document_ref` in metadata, signed by the ballot box
operator.

The blockchain record must be as close in time as practically possible to the creation of
the [Contest Ballot Checkpoint](contest_ballot_checkpoint.md) document.

<!-- markdownlint-disable max-one-sentence-per-line -->

```graphviz dot contest_ballot_checkpoint.dot.svg

{{ include_file('./../diagrams/contest_ballot_checkpoint.dot', indent=4) }}
```

<!-- markdownlint-enable max-one-sentence-per-line -->

### Validation

* The [`parameters`](../metadata.md#parameters) metadata *MUST* point to the Contest the ballot is being cast in.
* The 'ref' metadata fields reference the Contest Ballots collected in the proceeding
    period by the ballot box.
    These are sorted from earliest `document_id`:`document_ver` regardless of the time
    the individual ballot was received by the ballot box.
* Ballot boxes will not accept ballots whose `document_id`:`document_ver` fall outside
    the boundaries of the contest, or are not close in time to when the ballot box
    received the ballot.

### Business Logic

#### Front End

* This document is not produced by the Front End.
* The Front End may read the document to validate a given proof validates against a given
  `smt-root` and `smt-entries`.

#### Back End

* Validate the ballots being referenced exist and are valid for the contest.
* Signed by an authoritative Ballot Box.
* All referenced ballots are in the same contest as specified in the [`parameters`](../metadata.md#parameters) metadata.
* The Chain is intact and this document is consistent with the metadata in the previous checkpoint document.
* There is no previous checkpoint document which already references the same chained checkpoint document.

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

The Payload is a [CBOR][RFC8949] Document, and must conform to this schema.

It consists of an array which defines the weights to be applied to the chosen delegations.

Each valid delegate gets the matching weight from this array.
The total voting power is split proportionally based on these weights over the
valid drep nominations.

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

## Signers

The following Admin roles may sign documents of this type:

* Bulletin Board Operator

Only the original author can update and sign a new version of documents.

## Copyright

| Copyright | :copyright: 2024-2025 IOG Singapore, All Rights Reserved |
| --- | --- |
| License | This document is licensed under [CC-BY-4.0] |
| Created | 2024-12-27 |
| Modified | 2025-11-03 |
| Authors | Alex Pozhylenkov <alex.pozhylenkov@iohk.io> |
| | Nathan Bogale <nathan.bogale@iohk.io> |
| | Neil McAuliffe <neil.mcauliffe@iohk.io> |
| | Steven Johnson <steven.johnson@iohk.io> |

### Changelog

#### 0.1.5 (2025-11-03)

* Add Voting Ballots and Ballot Checkpoint Documents

[CBOR-TAG-42]: https://github.com/ipld/cid-cbor/
[RFC9052-HeaderParameters]: https://www.rfc-editor.org/rfc/rfc8152#section-3.1
[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
[IPFS-CID]: https://docs.ipfs.tech/concepts/content-addressing/#what-is-a-cid
[RFC9562-V7]: https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-7
[RFC8949]: https://www.rfc-editor.org/rfc/rfc8949.html
[RFC8610]: https://www.rfc-editor.org/rfc/rfc8610
