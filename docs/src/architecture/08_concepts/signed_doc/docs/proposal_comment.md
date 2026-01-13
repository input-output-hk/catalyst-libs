# Proposal Comment

## Description

A Proposal Comment is a document which comments on a referenced Proposal document.

Proposal Comments themselves are intentionally general, however they may be
linked to a brand/campaign or category via the template used by the proposal.

The payload of a proposal comment is controlled by its template.

<!-- markdownlint-disable max-one-sentence-per-line -->

```graphviz dot proposal_comment.dot.svg

{{ include_file('./../diagrams/proposal_comment.dot', indent=4) }}
```

<!-- markdownlint-enable max-one-sentence-per-line -->

### Validation

A comment which is a reply *MUST* reference the same document.
It may reference a different version of the document.

### Business Logic

#### Front End

Comments are valid for any version of the document, however
as comments refer to a specific version of a document, they may
lose context when displayed against the latest version of a document.
In these cases, the front end should clearly show that a comment was on
a different version of the document.

If the front end posts a reply to another comment:

* it should reference the comment being replied to in the [`reply`](../metadata.md#reply) field.
* The [`ref`](../metadata.md#ref) field must refer to the same document, but can be a different version.

#### Back End

The backend will only validate the document being referenced exists,
and the integrity of the [`ref`](../metadata.md#ref) and [`reply`](../metadata.md#reply) metadata fields is correct.

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
| Type | b679ded3-0e7c-41ba-89f8-da62a17898ea |
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

### [`template`](../metadata.md#template)

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Reference](../metadata.md#document-reference) |
| Valid References | [Proposal Comment Form Template](proposal_comment_form_template.md) |
<!-- markdownlint-enable MD033 -->
Reference to the template used to create and/or validate this document.

#### [`template`](../metadata.md#template) Validation

In addition to the validation performed for [Document Reference](../metadata.md#document-reference) type fields,
The document payload is not valid if it does not validate completely against the referenced template.

### [`reply`](../metadata.md#reply)

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | optional |
| Format | [Document Reference](../metadata.md#document-reference) |
| Valid References | [Proposal Comment](proposal_comment.md) |
<!-- markdownlint-enable MD033 -->
Reference to a Comment document type being referred to.

#### [`reply`](../metadata.md#reply) Validation

In addition to the validation performed for [Document Reference](../metadata.md#document-reference) type fields,
The [`ref`](../metadata.md#ref) of the [`reply`](../metadata.md#reply) document must be the same as
the original comment document.

### [`section`](../metadata.md#section)

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | optional |
| Format | [Section Reference](../metadata.md#section-reference) |
<!-- markdownlint-enable MD033 -->
A Reference to the original document, or the comment being replied to.

#### [`section`](../metadata.md#section) Validation

For a non-reply this must be a valid section reference into the referenced document.
For a reply, this must be a valid section reference into the comment being replied to.

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
| Valid References | [Brand Parameters](brand_parameters.md) |
|  | [Campaign Parameters](campaign_parameters.md) |
|  | [Category Parameters](category_parameters.md) |
| Linked Reference Metadata | [`ref`](#ref) |
|  | [`template`](#template) |
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

* The Document referenced by [`template`](../metadata.md#template)
    * MUST contain [`parameters`](../metadata.md#parameters) metadata; AND
    * MUST match the referencing documents [`parameters`](../metadata.md#parameters) value.

## Payload

[JSON][RFC8259] Document which must validate against the referenced template.

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

#### 0.01 (2025-04-04)

* First Published Version

#### 0.03 (2025-05-05)

* Use generalized parameters.

[CBOR-TAG-42]: https://github.com/ipld/cid-cbor/
[CBOR-LFD-ENCODING]: https://www.rfc-editor.org/rfc/rfc8949.html#section-4.2.3
[RFC9052-HeaderParameters]: https://www.rfc-editor.org/rfc/rfc8152#section-3.1
[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
[IPFS-CID]: https://docs.ipfs.tech/concepts/content-addressing/#what-is-a-cid
[RFC9562-V7]: https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-7
[RFC8259]: https://www.rfc-editor.org/rfc/rfc8259.html
