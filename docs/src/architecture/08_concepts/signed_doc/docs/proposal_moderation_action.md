# Proposal Moderation Action

> :warning: This section are still under development and may be updated or revised.
> These details are not yet confirmed and should not be used as the basis for any implementation.
> Proceeding with implementation based on this draft is at the implementer's own risk
> and may lead to incompatibility with the completed version.

## Description

A Moderation action performed on a Proposal.

<!-- markdownlint-disable max-one-sentence-per-line -->

```graphviz dot proposal_moderation_action.dot.svg

{{ include_file('./../diagrams/proposal_moderation_action.dot', indent=4) }}
```

<!-- markdownlint-enable max-one-sentence-per-line -->

### Validation

This specification outlines the required definitions for the current features.
The document will be incrementally improved in future iterations as more functionality
and features are added.
This section will be included and updated in future iterations.

### Business Logic

#### Front End

This specification outlines the required definitions for the current features.
The document will be incrementally improved in future iterations as more functionality
and features are added.
This section will be included and updated in future iterations.

#### Back End

This specification outlines the required definitions for the current features.
The document will be incrementally improved in future iterations as more functionality
and features are added.
This section will be included and updated in future iterations.

## [COSE Header Parameters][RFC9052-HeaderParameters]

* [content type](../spec.md#content-type) = `application/json`
* [content-encoding](../spec.md#content-encoding) = `[br]`

## Metadata

### [`type`](../metadata.md#type)

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Type](../metadata.md#document-type) |
| Type | a552451a-8e5b-409d-83a0-21eac26bbf8c |
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

## Payload

This specification outlines the required definitions for the current features.
The document will be incrementally improved in future iterations as more functionality
and features are added.
This section will be included and updated in future iterations.

## Signers

The following User roles may sign documents of this type:

* Registered

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

#### 0.01 (2025-04-04)

* First Published Version

[CBOR-TAG-42]: https://github.com/ipld/cid-cbor/
[CBOR-LFD-ENCODING]: https://www.rfc-editor.org/rfc/rfc8949.html#section-4.2.3
[RFC9052-HeaderParameters]: https://www.rfc-editor.org/rfc/rfc8152#section-3.1
[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
[IPFS-CID]: https://docs.ipfs.tech/concepts/content-addressing/#what-is-a-cid
[RFC9562-V7]: https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-7
