# Proposal Comment

## Description

A Proposal Comment is a document which comments on a referenced Proposal document.

Proposal Comments themselves are intentionally general, however they may be
linked to a brand/campaign or category via the template used by the proposal.

The payload of a proposal comment is controlled by its template.

```d2 layout="elk"
"Proposal Comment": {
  shape: sql_table
  "content type": application/json
  "type [0]": b679ded3-0e7c-41ba-89f8-da62a17898ea
  "type [1]": 7808d2ba-d511-40af-84e8-c0d1625fdfdc
  "id": UUIDv7
  "ver": UUIDv7
  "ref": Proposal
  "template": Proposal Comment Template
  "reply": Proposal Comment (Optional)
  "section": Section Reference
  "brand_id": Brand Parameters (Optional)
  "campaign_id": Campaign Parameters (Optional)
  "category_id": Category Parameters (Optional)

}

"Proposal Comment"."ref"->"Proposal"
"Proposal Comment"."template"->"Proposal Comment Template"
"Proposal Comment"."reply"->"Proposal Comment": <reply> Optional
"Proposal Comment"."brand_id"->"Brand Parameters": Optional
"Proposal Comment"."campaign_id"->"Campaign Parameters": Optional
"Proposal Comment"."category_id"->"Category Parameters": Optional
```

### Validation

A comment which is a reply *MUST* reference the same document.
It may reference a different version of the document.

### Business Logic

#### Front End

This specification outlines the required definitions for the current features.
The document will be incrementally improved in future iterations as more functionality
and features are added.
This section will be included and updated in future iterations.

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
| Type | `b679ded3-0e7c-41ba-89f8-da62a17898ea`,<br/>`7808d2ba-d511-40af-84e8-c0d1625fdfdc` |
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
| Valid References | [Proposal](proposal.md) |
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

### [`template`](../metadata.md#template)
<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Reference](../metadata.md#document-reference) |
| Valid References | [Proposal Comment Template](proposal_comment_template.md) |
<!-- markdownlint-enable MD033 -->
Reference to the template used to create and/or validate this document.

#### Validation

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

#### Validation

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

#### Validation

For a non-reply this must be a valid section reference into the referenced document.
For a reply, this must be a valid section reference into the comment being replied to.

### [`brand_id`](../metadata.md#brand_id)
<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | optional |
| Format | [Document Reference](../metadata.md#document-reference) |
| Valid References | [Brand Parameters](brand_parameters.md) |
| Linked Reference Metadata | [`ref`](#ref) |
|  | [`template`](#template) |
| Exclusive | [`campaign_id`](../metadata.md#campaign_id) |
|  | [`category_id`](../metadata.md#category_id) |
<!-- markdownlint-enable MD033 -->
A reference to the Brand Parameters Document this document lies under.

#### Validation

In addition to the validation performed for [Document Reference](../metadata.md#document-reference) type fields:

* Any linked referenced document that includes a [`brand_id`](../metadata.md#brand_id) must match the
[`brand_id`](../metadata.md#brand_id) of the referencing document.
* MUST NOT be present in any document that contains
[`campaign_id`](../metadata.md#campaign_id)
and [`category_id`](../metadata.md#category_id) metadata.
* The Document referenced by [`ref`](../metadata.md#ref)
  * MUST contain [`brand_id`](../metadata.md#brand_id) metadata; AND
  * MUST match the referencing documents [`brand_id`](../metadata.md#brand_id) value.
* The Document referenced by [`template`](../metadata.md#template)
  * MUST contain [`brand_id`](../metadata.md#brand_id) metadata; AND
  * MUST match the referencing documents [`brand_id`](../metadata.md#brand_id) value.

### [`campaign_id`](../metadata.md#campaign_id)
<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | optional |
| Format | [Document Reference](../metadata.md#document-reference) |
| Valid References | [Campaign Parameters](campaign_parameters.md) |
| Linked Reference Metadata | [`ref`](#ref) |
|  | [`template`](#template) |
| Exclusive | [`brand_id`](../metadata.md#brand_id) |
|  | [`category_id`](../metadata.md#category_id) |
<!-- markdownlint-enable MD033 -->
A reference to the Campaign Parameters Document this document lies under.

#### Validation

In addition to the validation performed for [Document Reference](../metadata.md#document-reference) type fields:

* Any linked referenced document that includes a [`campaign_id`](../metadata.md#campaign_id) must match the
[`campaign_id`](../metadata.md#campaign_id) of the referencing document.
* MUST NOT be present in any document that contains
[`brand_id`](../metadata.md#brand_id)
and [`category_id`](../metadata.md#category_id) metadata.
* The Document referenced by [`ref`](../metadata.md#ref)
  * MUST contain [`campaign_id`](../metadata.md#campaign_id) metadata; AND
  * MUST match the referencing documents [`campaign_id`](../metadata.md#campaign_id) value.
* The Document referenced by [`template`](../metadata.md#template)
  * MUST contain [`campaign_id`](../metadata.md#campaign_id) metadata; AND
  * MUST match the referencing documents [`campaign_id`](../metadata.md#campaign_id) value.

### [`category_id`](../metadata.md#category_id)
<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | optional |
| Format | [Document Reference](../metadata.md#document-reference) |
| Valid References | [Category Parameters](category_parameters.md) |
| Linked Reference Metadata | [`ref`](#ref) |
|  | [`template`](#template) |
| Exclusive | [`brand_id`](../metadata.md#brand_id) |
|  | [`campaign_id`](../metadata.md#campaign_id) |
<!-- markdownlint-enable MD033 -->
A reference to the Category Parameters Document this document lies under.

#### Validation

In addition to the validation performed for [Document Reference](../metadata.md#document-reference) type fields:

* Any linked referenced document that includes a [`category_id`](../metadata.md#category_id) must match the
[`category_id`](../metadata.md#category_id) of the referencing document.
* MUST NOT be present in any document that contains
[`brand_id`](../metadata.md#brand_id)
and [`campaign_id`](../metadata.md#campaign_id) metadata.
* The Document referenced by [`ref`](../metadata.md#ref)
  * MUST contain [`category_id`](../metadata.md#category_id) metadata; AND
  * MUST match the referencing documents [`category_id`](../metadata.md#category_id) value.
* The Document referenced by [`template`](../metadata.md#template)
  * MUST contain [`category_id`](../metadata.md#category_id) metadata; AND
  * MUST match the referencing documents [`category_id`](../metadata.md#category_id) value.

## Payload

[JSON][RFC8259] Document which must validate against the referenced template.

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
[RFC8259]: https://www.rfc-editor.org/rfc/rfc8259.html
