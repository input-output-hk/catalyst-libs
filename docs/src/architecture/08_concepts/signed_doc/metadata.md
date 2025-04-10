# Metadata Fields

## Metadata Types

The following types of metadata have been defined.
All Metadata fields use one of these types.

### Collaborators Reference List

A list of collaborators who can participate in drafting and submitting a document

#### [CDDL][RFC8610] Specification

```cddl
catalyst_id = text
collaborators = [ * catalyst_id ]
```

### Document Reference

A document reference identifier

#### [CDDL][RFC8610] Specification

```cddl
uuid_v7 = 6.37(bytes .size 16)
document_id = uuid_v7
document_ver = uuid_v7
blake2b_256 = bytes .size 32
document_hash = blake2b_256
document_ref = [ 1* [ document_id, document_ver, document_hash ] ]
```

### Document Type

A document type identifier

#### [CDDL][RFC8610] Specification

```cddl
uuid_v4 = 6.37(bytes .size 16)
document_type = [ 1* uuid_v4 ]
```

### Section Reference

A document section reference identifier

#### [CDDL][RFC8610] Specification

```cddl
json_pointer = text
section_ref = json_pointer
```

### [UUIDv4][RFC9562-V4]

Version 4 formatted [UUID][RFC9562]

#### [CDDL][RFC8610] Specification

```cddl
uuid_v4 = 6.37(bytes .size 16)
```

### [UUIDv7][RFC9562-V7]

Version 7 formatted [UUID][RFC9562]

#### [CDDL][RFC8610] Specification

```cddl
uuid_v7 = 6.37(bytes .size 16)
```

## Individual Metadata field definitions

### `type`
<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Type](metadata.md#document-type) |
<!-- markdownlint-enable MD033 -->
The document TYPE.

#### Validation

**MUST** be a known document type.

### `id`
<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [UUIDv7](metadata.md#uuidv7) |
<!-- markdownlint-enable MD033 -->
Document ID, created the first time the document is created.
This must be a properly created [UUIDv7][RFC9562-V7] which contains the
timestamp of when the document was created.

#### Validation

IF [`ver`](metadata.md#ver) does not == [`id`](metadata.md#id) then a document with
[`id`](metadata.md#id) and [`ver`](metadata.md#ver) being equal *MUST* exist.

### `ver`
<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [UUIDv7](metadata.md#uuidv7) |
<!-- markdownlint-enable MD033 -->
The unique version of the document.
The first version of the document must set [`ver`](metadata.md#ver) == [`id`](metadata.md#id)

#### Validation

The document version must always be >= the document ID.

### `ref`
<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | optional |
| Format | [Document Reference](metadata.md#document-reference) |
| Valid References | [Proposal Meta Template](./docs/proposal_meta_template.md) |
|  | [Proposal Template](./docs/proposal_template.md) |
|  | [Proposal](./docs/proposal.md) |
|  | [Proposal Comment Meta Template](./docs/proposal_comment_meta_template.md) |
|  | [Proposal Comment Template](./docs/proposal_comment_template.md) |
|  | [Proposal Comment](./docs/proposal_comment.md) |
|  | [Proposal Submission Action](./docs/proposal_submission_action.md) |
|  | [Proposal Moderation Action](./docs/proposal_moderation_action.md) |
|  | [Comment Moderation Action](./docs/comment_moderation_action.md) |
|  | [Brand Parameters](./docs/brand_parameters.md) |
|  | [Campaign Parameters](./docs/campaign_parameters.md) |
|  | [Category Parameters](./docs/category_parameters.md) |
|  | [Election Parameters](./docs/election_parameters.md) |
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

### `template`
<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | optional |
| Format | [Document Reference](metadata.md#document-reference) |
| Valid References | [Proposal Meta Template](./docs/proposal_meta_template.md) |
|  | [Proposal Template](./docs/proposal_template.md) |
|  | [Proposal Comment Meta Template](./docs/proposal_comment_meta_template.md) |
|  | [Proposal Comment Template](./docs/proposal_comment_template.md) |
<!-- markdownlint-enable MD033 -->
Reference to the template used to create and/or validate this document.

#### Validation

In addition to the validation performed for [Document Reference](metadata.md#document-reference) type fields,
The document payload is not valid if it does not validate completely against the referenced template.

### `reply`
<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | optional |
| Format | [Document Reference](metadata.md#document-reference) |
| Valid References | [Proposal Comment](./docs/proposal_comment.md) |
<!-- markdownlint-enable MD033 -->
Reference to a Comment document type being referred to.

#### Validation

In addition to the validation performed for [Document Reference](metadata.md#document-reference) type fields,
The [`ref`](metadata.md#ref) of the [`reply`](metadata.md#reply) document must be the same as
the original comment document.

### `section`
<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | optional |
| Format | [Section Reference](metadata.md#section-reference) |
<!-- markdownlint-enable MD033 -->
A Reference to the original document, or the comment being replied to.

#### Validation

For a non-reply this must be a valid section reference into the referenced document.
For a reply, this must be a valid section reference into the comment being replied to.

### `collaborators`
<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | optional |
| Format | [Collaborators Reference List](metadata.md#collaborators-reference-list) |
<!-- markdownlint-enable MD033 -->
A list of collaborators who may also publish updates to versions of this document.
This should include all parties who have not signed this document directly.

Every subsequent version can amend the collaborators list.
However, the initial Author can never be removed from being able to
publish a new version of the document.

#### Validation

This list does not imply these collaborators have consented to collaborate, only that the author/s
are permitting these potential collaborators to participate in the drafting and submission process.
However, any document submission referencing a proposal MUST be signed by all collaborators in
addition to the author.

### `brand_id`
<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | optional |
| Format | [Document Reference](metadata.md#document-reference) |
| Valid References | [Brand Parameters](./docs/brand_parameters.md) |
| Exclusive | [`campaign_id`](metadata.md#campaign_id) |
|  | [`category_id`](metadata.md#category_id) |
<!-- markdownlint-enable MD033 -->
A reference to the Brand Parameters Document this document lies under.

#### Validation

In addition to the validation performed for [Document Reference](metadata.md#document-reference) type fields:

* Any linked referenced document that includes a [`brand_id`](metadata.md#brand_id) must match the
[`brand_id`](metadata.md#brand_id) of the referencing document.
* MUST NOT be present in any document that contains
[`campaign_id`](metadata.md#campaign_id)
and [`category_id`](metadata.md#category_id) metadata.

### `campaign_id`
<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | optional |
| Format | [Document Reference](metadata.md#document-reference) |
| Valid References | [Campaign Parameters](./docs/campaign_parameters.md) |
| Exclusive | [`brand_id`](metadata.md#brand_id) |
|  | [`category_id`](metadata.md#category_id) |
<!-- markdownlint-enable MD033 -->
A reference to the Campaign Parameters Document this document lies under.

#### Validation

In addition to the validation performed for [Document Reference](metadata.md#document-reference) type fields:

* Any linked referenced document that includes a [`campaign_id`](metadata.md#campaign_id) must match the
[`campaign_id`](metadata.md#campaign_id) of the referencing document.
* MUST NOT be present in any document that contains
[`brand_id`](metadata.md#brand_id)
and [`category_id`](metadata.md#category_id) metadata.

### `category_id`
<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | optional |
| Format | [Document Reference](metadata.md#document-reference) |
| Valid References | [Category Parameters](./docs/category_parameters.md) |
| Exclusive | [`brand_id`](metadata.md#brand_id) |
|  | [`campaign_id`](metadata.md#campaign_id) |
<!-- markdownlint-enable MD033 -->
A reference to the Category Parameters Document this document lies under.

#### Validation

In addition to the validation performed for [Document Reference](metadata.md#document-reference) type fields:

* Any linked referenced document that includes a [`category_id`](metadata.md#category_id) must match the
[`category_id`](metadata.md#category_id) of the referencing document.
* MUST NOT be present in any document that contains
[`brand_id`](metadata.md#brand_id)
and [`campaign_id`](metadata.md#campaign_id) metadata.

### `election_id`
<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | optional |
| Format | [Document Reference](metadata.md#document-reference) |
| Valid References | [Election Parameters](./docs/election_parameters.md) |
<!-- markdownlint-enable MD033 -->
A reference to the Election Parameters Document this document lies under.

#### Validation

In addition to the validation performed for [Document Reference](metadata.md#document-reference) type fields,
Any referenced document that includes a [`election_id`](metadata.md#election_id) must match the
[`election_id`](metadata.md#election_id) of the referencing document.
It is also valid for the referenced document to not include this field, if it is
optional for the referenced document.

## Copyright

| Copyright | :copyright: 2024-2025 IOG Singapore, All Rights Reserved |
| --- | --- |
| License | This document is licensed under [CC-BY-4.0] |
| Created | 2024-12-27 |
| Modified | 2025-04-09 |
| Authors | Alex Pozhylenkov <alex.pozhylenkov@iohk.io> |
| | Steven Johnson <steven.johnson@iohk.io> |

[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
[RFC9562-V4]: https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-4
[RFC9562-V7]: https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-7
[RFC8610]: https://www.rfc-editor.org/rfc/rfc8610
[RFC9562]: https://www.rfc-editor.org/rfc/rfc9562.html
