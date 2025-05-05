# Metadata Fields

## Metadata Types

The following types of metadata have been defined.
All Metadata fields use one of these types.

### Collaborators Reference List

A list of collaborators who can participate in drafting and submitting a document

<!-- markdownlint-disable max-one-sentence-per-line -->
??? note "CDDL Specification"

    * [cddl/collaborators.cddl](cddl/collaborators.cddl)

    ```cddl
    {{ include_file('./cddl/collaborators.cddl', indent=4) }}
    ```

<!-- markdownlint-enable max-one-sentence-per-line -->

### Document Id

A unique document identifier

<!-- markdownlint-disable max-one-sentence-per-line -->
??? note "CDDL Specification"

    * [cddl/document_id.cddl](cddl/document_id.cddl)

    ```cddl
    {{ include_file('./cddl/document_id.cddl', indent=4) }}
    ```

<!-- markdownlint-enable max-one-sentence-per-line -->

### Document Reference

A document reference identifier

<!-- markdownlint-disable max-one-sentence-per-line -->
??? note "CDDL Specification"

    * [cddl/document_ref.cddl](cddl/document_ref.cddl)

    ```cddl
    {{ include_file('./cddl/document_ref.cddl', indent=4) }}
    ```

<!-- markdownlint-enable max-one-sentence-per-line -->

### Document Type

A document type identifier

<!-- markdownlint-disable max-one-sentence-per-line -->
??? note "CDDL Specification"

    * [cddl/document_type.cddl](cddl/document_type.cddl)

    ```cddl
    {{ include_file('./cddl/document_type.cddl', indent=4) }}
    ```

<!-- markdownlint-enable max-one-sentence-per-line -->

### Document Ver

A unique chronological document version

<!-- markdownlint-disable max-one-sentence-per-line -->
??? note "CDDL Specification"

    * [cddl/document_ver.cddl](cddl/document_ver.cddl)

    ```cddl
    {{ include_file('./cddl/document_ver.cddl', indent=4) }}
    ```

<!-- markdownlint-enable max-one-sentence-per-line -->

### Section Reference

A document section reference identifier

<!-- markdownlint-disable max-one-sentence-per-line -->
??? note "CDDL Specification"

    * [cddl/section_ref.cddl](cddl/section_ref.cddl)

    ```cddl
    {{ include_file('./cddl/section_ref.cddl', indent=4) }}
    ```

<!-- markdownlint-enable max-one-sentence-per-line -->

### [UUIDv4][RFC9562-V4]

Version 4 formatted [UUID][RFC9562]

<!-- markdownlint-disable max-one-sentence-per-line -->
??? note "CDDL Specification"

    * [cddl/uuid_v4.cddl](cddl/uuid_v4.cddl)

    ```cddl
    {{ include_file('./cddl/uuid_v4.cddl', indent=4) }}
    ```

<!-- markdownlint-enable max-one-sentence-per-line -->

### [UUIDv7][RFC9562-V7]

Version 7 formatted [UUID][RFC9562]

<!-- markdownlint-disable max-one-sentence-per-line -->
??? note "CDDL Specification"

    * [cddl/uuid_v7.cddl](cddl/uuid_v7.cddl)

    ```cddl
    {{ include_file('./cddl/uuid_v7.cddl', indent=4) }}
    ```

<!-- markdownlint-enable max-one-sentence-per-line -->

### Version Revocations

A list of all versions of this document which are 'revoked'.

<!-- markdownlint-disable max-one-sentence-per-line -->
??? note "CDDL Specification"

    * [cddl/revocations.cddl](cddl/revocations.cddl)

    ```cddl
    {{ include_file('./cddl/revocations.cddl', indent=4) }}
    ```

<!-- markdownlint-enable max-one-sentence-per-line -->

## Individual Metadata field definitions

### `type`

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Type](metadata.md#document-type) |
<!-- markdownlint-enable MD033 -->
The document TYPE.

#### `type` Validation

**MUST** be a known document type.

### `id`

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Id](metadata.md#document-id) |
<!-- markdownlint-enable MD033 -->
Document ID, created the first time the document is created.
This must be a properly created [UUIDv7][RFC9562-V7] which contains the
timestamp of when the document was created.

#### `id` Validation

IF [`ver`](metadata.md#ver) does not == [`id`](metadata.md#id) then a document with
[`id`](metadata.md#id) and [`ver`](metadata.md#ver) being equal *MUST* exist.

### `ver`

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Ver](metadata.md#document-ver) |
<!-- markdownlint-enable MD033 -->
The unique version of the document.
The first version of the document must set [`ver`](metadata.md#ver) == [`id`](metadata.md#id)

#### `ver` Validation

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

#### `ref` Validation

The following must be true for a valid reference:

* The Referenced Document **MUST** Exist
* Every value in the `document_locator` must consistently reference the exact same document.
* The `document_id` and `document_ver` **MUST** match the values in the referenced document.

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

#### `template` Validation

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

#### `reply` Validation

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

#### `section` Validation

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

#### `collaborators` Validation

This list does not imply these collaborators have consented to collaborate, only that the author/s
are permitting these potential collaborators to participate in the drafting and submission process.
However, any document submission referencing a proposal MUST be signed by all collaborators in
addition to the author.

### `revocations`

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | excluded |

### `parameters`

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | optional |
| Format | [Document Reference](metadata.md#document-reference) |
| Valid References | [Brand Parameters](./docs/brand_parameters.md) |
|  | [Campaign Parameters](./docs/campaign_parameters.md) |
|  | [Category Parameters](./docs/category_parameters.md) |
|  | [Election Parameters](./docs/election_parameters.md) |
<!-- markdownlint-enable MD033 -->
A reference to the Parameters Document this document lies under.

#### `parameters` Validation

In addition to the validation performed for [Document Reference](metadata.md#document-reference) type fields:

* Any linked referenced document that includes a [`parameters`](metadata.md#parameters) metadata must match the
[`parameters`](metadata.md#parameters) of the referencing document.

## Copyright

| Copyright | :copyright: 2024-2025 IOG Singapore, All Rights Reserved |
| --- | --- |
| License | This document is licensed under [CC-BY-4.0] |
| Created | 2024-12-27 |
| Modified | 2025-05-05 |
| Authors | Alex Pozhylenkov <alex.pozhylenkov@iohk.io> |
| | Steven Johnson <steven.johnson@iohk.io> |

[CBOR-TAG-42]: https://github.com/ipld/cid-cbor/
[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
[IPFS-CID]: https://docs.ipfs.tech/concepts/content-addressing/#what-is-a-cid
[RFC9562-V4]: https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-4
[RFC9562-V7]: https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-7
[RFC9562]: https://www.rfc-editor.org/rfc/rfc9562.html
