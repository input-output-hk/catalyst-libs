# Comment Moderation Action

## Description

A Moderation Action performed on any Comment.

```d2 layout="elk"
"Comment Moderation Action": {
  shape: sql_table
  "content type": application/json
  "type [0]": 5e60e623-ad02-4a1b-a1ac-406db978ee48
  "type [1]": b679ded3-0e7c-41ba-89f8-da62a17898ea
  "type [2]": a5d232b8-5e03-4117-9afd-be32b878fcdd
  "id": UUIDv7
  "ver": UUIDv7
  "ref": Proposal Comment

}

"Comment Moderation Action"."ref"->"Proposal Comment"
```

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
| Type | `5e60e623-ad02-4a1b-a1ac-406db978ee48`,<br/>`b679ded3-0e7c-41ba-89f8-da62a17898ea`,<br/>`a5d232b8-5e03-4117-9afd-be32b878fcdd` |
<!-- markdownlint-enable MD033 -->
The document TYPE.

#### Validation

**MUST** be a known document type.

### [`id`](../metadata.md#id)
<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [UUIDv7](../metadata.md#uuidv7) |
<!-- markdownlint-enable MD033 -->
Document ID, created the first time the document is created.
This must be a properly created [UUIDv7][RFC9562-V7] which contains the
timestamp of when the document was created.

#### Validation

IF [`ver`](../metadata.md#ver) does not == [`id`](../metadata.md#id) then a document with
[`id`](../metadata.md#id) and [`ver`](../metadata.md#ver) being equal *MUST* exist.

### [`ver`](../metadata.md#ver)
<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [UUIDv7](../metadata.md#uuidv7) |
<!-- markdownlint-enable MD033 -->
The unique version of the document.
The first version of the document must set [`ver`](../metadata.md#ver) == [`id`](../metadata.md#id)

#### Validation

The document version must always be >= the document ID.

### [`ref`](../metadata.md#ref)
<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Reference](../metadata.md#document-reference) |
| Valid References | [Proposal Comment](proposal_comment.md) |
<!-- markdownlint-enable MD033 -->
Reference to a Linked Document or Documents.
This is the primary hierarchical reference to a related document.

This is an Array of the format:
    `[[DocumentID, DocumentVer, DocumentHash],...]`

* `DocumentID` is the [UUIDv7][RFC9562-V7] ID of the Document being referenced.
* `DocumentVer` is the [UUIDv7][RFC9562-V7] Version of the Document being referenced.
* `DocumentHash` is the Blake2b-256 Hash of the entire document being referenced, not just its payload.
  It ensures that the intended referenced document is the one used, and there has been no substitution.
  Prevents substitutions where a new document with the same Document ID and Ver might be published over an existing one.

#### Validation

Every Reference Document **MUST** Exist, and **MUST** be a valid reference to the document.
The calculated Hash of the Referenced Document **MUST** match the Hash in the reference.

## Payload

TODO

## Signers

The following user roles may sign documents of this type:

* Registered

New versions of this document may be published by:

* author

## Copyright

| Copyright | :copyright: 2024-2025 IOG Singapore, All Rights Reserved |
| --- | --- |
| License | This document is licensed under [CC-BY-4.0] |
| Created | 2024-12-27 |
| Modified | 2025-04-09 |
| Authors | Alex Pozhylenkov <alex.pozhylenkov@iohk.io> |
| | Steven Johnson <steven.johnson@iohk.io> |

### Changelog

#### 0.01 (2025-04-04)

* First Published Version

[RFC9052-HeaderParameters]: https://www.rfc-editor.org/rfc/rfc8152#section-3.1
[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
[RFC9562-V7]: https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-7
