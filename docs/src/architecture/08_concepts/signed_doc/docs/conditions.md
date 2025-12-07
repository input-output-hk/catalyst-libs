# Conditions

> **DRAFT STATUS**  
> This document is currently in **DRAFT** status. Development should **NOT** begin until this specification is formally released.  
> This specification is subject to change without notice.

## Description

Conditions documents are simple document types published by authoritative parties (such as Catalyst admins) to define terms and conditions that users must accept before submitting documents to the system.

The Conditions document type supports multiple condition documents for different purposes, such as:

* Terms of Use (TOU)
* License agreements
* Operational guidelines
* Regional restrictions
* Privacy policies
* Other legal or operational requirements

The payload of a Conditions document contains the text of the terms and conditions, typically in Markdown or HTML format. This allows for rich formatting while maintaining human readability.

Conditions documents are versioned and can be revoked, enabling administrators to update terms over time while maintaining an auditable history of what terms were in effect at any given time.

Conditions documents are referenced by parameter documents (Brand, Campaign, Category, and Contest Parameters) to specify which conditions are required at each level of the system hierarchy. User-submitted documents (such as Proposals and Comments) must reference all required conditions from their parameter hierarchy, with the act of listing these references and signing the document serving as the user's digital signature and acceptance.

<!-- markdownlint-disable max-one-sentence-per-line -->

```graphviz dot conditions.dot.svg

{{ include_file('./../diagrams/conditions.dot', indent=4) }}
```

<!-- markdownlint-enable max-one-sentence-per-line -->

### Validation

The Conditions document *MUST* be a valid signed document according to the Signed Document Standard.

When a Conditions document is referenced in a parameter document's [`conditions`](../metadata.md#conditions) metadata field, the referenced document *MUST* exist and be of type "Conditions".

When a Conditions document is referenced in a user-submitted document's [`conditions`](../metadata.md#conditions) metadata field, the referenced document *MUST* exist, be of type "Conditions", and not be revoked.

### Business Logic

#### Front End

Front-end applications should:

* Display Conditions documents to users when they are required to accept them
* Store user acceptance locally to minimize friction (users only need to explicitly accept conditions the first time they encounter them)
* Gray out submission buttons until all required conditions have been accepted
* Display a disclosure on submission listing all accepted conditions under which the document is being submitted
* Provide clear error messages if required conditions are missing or invalid

#### Back End

Back-end validation must:

* Verify that all Conditions documents referenced in user-submitted documents exist and are valid
* Collect all required conditions from the parameter hierarchy (Brand → Campaign → Category → Contest)
* Ensure user-submitted documents include exactly the union of all required conditions from their parameter hierarchy
* Reject documents that reference revoked Conditions documents
* Reject documents that are missing required conditions or include conditions not in the parameter hierarchy

The decentralized system (Hermes) will also reject documents without the required conditions, ensuring validation occurs at multiple layers.

## [COSE Header Parameters][RFC9052-HeaderParameters]

* [content type](../spec.md#content-type) = `text/markdown` or `text/html`
* [content-encoding](../spec.md#content-encoding) = `[br]`

## Metadata

### [`type`](../metadata.md#type)

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Type](../metadata.md#document-type) |
| Type | b664afc2-6472-4028-b90f-875bf6eefab8 |
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
| Required | optional |
| Format | [Document Reference](../metadata.md#document-reference) |
<!-- markdownlint-enable MD033 -->
Reference to a Linked Document or Documents.

This field may be used to reference related documents, such as:
* Related Conditions documents (e.g., a privacy policy that references a terms of use)
* Legal documents or regulations that the conditions are based on
* Other relevant documentation

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

**Important**: User-submitted documents that reference revoked Conditions documents *MUST* be rejected during validation.

## Payload

The Conditions document payload contains the text of the terms and conditions.

The payload *MUST* be valid according to the content type specified in the COSE header:

* If `content-type` is `text/markdown`, the payload must be valid Markdown
* If `content-type` is `text/html`, the payload must be valid HTML5

The payload is compressed using Brotli compression (`br` encoding) as specified in the `content-encoding` header.

The payload content should be human-readable and clearly state:
* The purpose of the conditions
* What users are agreeing to
* Any obligations or restrictions
* Effective dates or version information
* Contact information for questions

## Signers

The following Admin roles may sign documents of this type:

* Brand Admin
* Campaign Admin
* Category Admin
* Contest Admin

The specific admin role required depends on the level at which the Conditions document is intended to be used. For example:
* Brand-level conditions should be signed by Brand Admin
* Campaign-level conditions should be signed by Campaign Admin or Brand Admin
* Category-level conditions should be signed by Category Admin or a parent-level admin
* Contest-level conditions should be signed by Contest Admin or a parent-level admin

Updates are allowed by the original author and from the 'collaborators' metadata field
of the previous submitted document's version.

## JSON Specification Requirements

> **DRAFT STATUS**  
> The following JSON specification changes are documented here but **MUST NOT** be implemented until this specification is formally released.  
> The `signed_doc.json` file is the source of truth for code generation and must be marked as `draft: true` when these changes are added.

### Required Changes to `signed_doc.json`

The following changes must be made to `catalyst-libs/specs/signed_doc.json`:

#### 1. Add "Conditions" Document Type

Add a new entry to the `docs` section with the following structure:

```json
"Conditions": {
  "authors": {
    "Nathan Bogale": "nathan.bogale@iohk.io",
    "Steven Johnson": "steven.johnson@iohk.io"
  },
  "description": "Conditions documents define terms and conditions that users must accept before submitting documents to the system. Supports multiple condition types (TOU, license agreements, operational guidelines, regional restrictions).",
  "draft": true,
  "type": "<UUIDv4 to be generated>",
  "headers": {
    "content type": {
      "coseLabel": 3,
      "description": "Media Type/s allowed in the Payload",
      "format": "Media Type",
      "required": "yes",
      "value": "text/markdown"
    },
    "content-encoding": {
      "coseLabel": "content-encoding",
      "description": "Supported HTTP Encodings of the Payload",
      "format": "HTTP Content Encoding",
      "required": "optional",
      "value": ["br"]
    }
  },
  "metadata": {
    "type": {
      "description": "The document TYPE.",
      "format": "Document Type",
      "required": "yes",
      "validation": "**MUST** be a known document type."
    },
    "id": {
      "description": "Document ID, created the first time the document is created.",
      "format": "Document Id",
      "required": "yes"
    },
    "ver": {
      "description": "The unique version of the document.",
      "format": "Document Ver",
      "required": "yes"
    },
    "ref": {
      "description": "Reference to a Linked Document or Documents.",
      "format": "Document Reference",
      "required": "optional"
    },
    "revocations": {
      "description": "A document may include a list of any prior versions which are considered to be revoked.",
      "format": "Version Revocations",
      "required": "optional"
    }
  },
  "payload": {
    "description": "The Conditions document payload contains the text of the terms and conditions in Markdown or HTML format.",
    "nil": false
  },
  "signers": {
    "roles": {
      "admin": ["Brand Admin", "Campaign Admin", "Category Admin", "Contest Admin"],
      "user": []
    },
    "update": {
      "description": "Updates are allowed by the original author and from the 'collaborators' metadata field of the previous submitted document's version.",
      "type": "author"
    }
  },
  "validation": "The Conditions document *MUST* be a valid signed document according to the Signed Document Standard.",
  "versions": [
    {
      "version": "0.01",
      "modified": "2025-01-XX",
      "changes": "* First Published Version (DRAFT)"
    }
  ]
}
```

**Important Notes:**
- Generate a new UUIDv4 for the `type` field
- Set `draft: true` as required by Steven Johnson
- The `value` for `content type` may be `text/markdown` or `text/html` (consider supporting both)

#### 2. Add "conditions" Metadata Field

Add the `conditions` metadata field to the following document types in their `metadata` sections:

**Parameter Documents** (Brand Parameters, Campaign Parameters, Category Parameters, Contest Parameters):
```json
"conditions": {
  "description": "An array of references to Conditions documents that define terms and conditions required at this level.",
  "format": "Document Reference",
  "required": "optional",
  "multiple": true,
  "type": ["Conditions"],
  "validation": "If present, must be an array of valid Conditions document references. All referenced documents must exist and be of type 'Conditions'. The array must be sorted according to CBOR Deterministic Encoding rules."
}
```

**User-Submitted Documents** (Proposal, Proposal Comment, etc.):
```json
"conditions": {
  "description": "An array of references to all Conditions documents that the user has accepted. Must include ALL conditions required by the parameter hierarchy (Brand → Campaign → Category → Contest).",
  "format": "Document Reference",
  "required": "optional",
  "multiple": true,
  "type": ["Conditions"],
  "validation": "Must exactly match the union of all required conditions from the parameter hierarchy. All referenced documents must exist, be valid, and not be revoked. The array must be sorted according to CBOR Deterministic Encoding rules."
}
```

**Important Notes:**
- Set `multiple: true` to allow arrays of document references
- Set `required: "optional"` (but required when conditions are specified in parameter hierarchy for user documents)
- The validation logic for user documents requires transitive collection from parameter hierarchy

#### 3. Update Document Types Table

The `types.md` file will be automatically regenerated from the JSON specification once these changes are made. Ensure the Conditions document type appears in the generated table.

### Code Generation Impact

Once these changes are made to `signed_doc.json` and the specification is released (draft status removed), the following code will need to be updated:

- **Rust**: Add `conditions` field to metadata structs and implement validation rules
- **Python**: Add `conditions` to `DocType` enum and metadata handling
- **Dart**: Add `conditions` to `DocumentType` enum
- **Backend Validation**: Implement transitive condition collection and matching logic
- **CLI Tools**: Add condition querying and acceptance prompts

## Copyright

| Copyright | :copyright: 2024-2025 IOG Singapore, All Rights Reserved |
| --- | --- |
| License | This document is licensed under [CC-BY-4.0] |
| Created | 2025-01-XX |
| Modified | 2025-01-XX |
| | Nathan Bogale <nathan.bogale@iohk.io> |
| | Steven Johnson <steven.johnson@iohk.io> |

### Changelog

#### 0.01 (2025-01-XX)

* First Published Version (DRAFT)

[CBOR-LFD-ENCODING]: https://www.rfc-editor.org/rfc/rfc8949.html#section-4.2.3
[RFC9052-HeaderParameters]: https://www.rfc-editor.org/rfc/rfc8152#section-3.1
[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
[RFC9562-V7]: https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-7

