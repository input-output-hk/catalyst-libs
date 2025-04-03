# Proposal Submission Action

## Description

## Proposal Submission Action

A Proposal Submission Action is a document which can attempt to either submit a
particular version of a proposal into a campaign, or withdraw it.

The last action on the document ts the action which takes effect at the deadline.

For multiple collaborators, multiple submission actions can be posted independently,
but none of them will take effect until ALL collaborators have posted equivalent actions.

For example, three collaborators Alice/Bob/Claire can each post one submission action
for the same document.
Unless they all submit the same version of the proposal
the proposal will not be seen as submitted.

The payload is a fixed format.

```d2 layout=elk

"Proposal Submission Action": {
  shape: sql_table
  "content type": [application/json]
  "type [0]": 5e60e623-ad02-4a1b-a1ac-406db978ee48
  "type [1]": 7808d2ba-d511-40af-84e8-c0d1625fdfdc
  "type [2]": 78927329-cfd9-4ea1-9c71-0e019b126a65
  "id": [UUIDv7][RFC9562-V7]
  "ver": [UUIDv7][RFC9562-V7]
  "ref": Proposal
  "category_id": Category Parameters

}

"Proposal Submission Action"."ref"->"Proposal"
"Proposal Submission Action"."category_id"->"Category Parameters"


```

### Validation

TODO

### Business Logic

#### Front End

TODO

#### Back End

TODO

## [COSE Header Parameters][RFC9052-HeaderParameters]

* [content type](../spec.md#content-type) = `application/json`
* [content-encoding](../spec.md#content-encoding) = `[br]`

## Metadata

### `type`
<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Type](../metadata.md#document-type) |
| Type | `5e60e623-ad02-4a1b-a1ac-406db978ee48`,<br/>`7808d2ba-d511-40af-84e8-c0d1625fdfdc`,<br/>`78927329-cfd9-4ea1-9c71-0e019b126a65` |
<!-- markdownlint-enable MD033 -->
The document TYPE.

#### Validation

**MUST** be a known document type.

### `id`
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

### `ver`
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

### `ref`
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

### `category_id`
<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Reference](../metadata.md#document-reference) |
| Valid References | [Category Parameters](category_parameters.md) |
| Exclusive |  brand_id  |
|  |  campaign_id  |
<!-- markdownlint-enable MD033 -->
A reference to the Category Parameters Document this document lies under.

#### Validation

In addition to the validation performed for `ref`,
Any referenced document that includes a [`category_id`](../metadata.md#category_id) must match the
[`category_id`](../metadata.md#category_id) of the referencing document.
It is also valid for the referenced document to not include this field, if it is
optional for the referenced document.

## Payload

The kind of action is controlled by this payload.
The Payload is a [JSON][RFC8259] Document, and must conform to this schema.

States:

* `final` : All collaborators must publish a `final` status for the proposal to be `final`.
* `draft` : Reverses the previous `final` state for a signer.
* `hide`  : Requests the proposal be hidden (not final, but a hidden draft).
        `hide` is only actioned if sent by the author, for a collaborator its synonymous with `draft`.

Schema :
<!-- markdownlint-disable MD013 -->
```json
{
  "$id": "https://raw.githubusercontent.com/input-output-hk/catalyst-libs/refs/heads/main/specs/signed_docs/docs/payload_schemas/proposal_submission_action.schema.json",
  "$schema": "http://json-schema.org/draft-07/schema#",
  "additionalProperties": false,
  "definitions": {
    "action": {
      "description": "The action being performed on the Proposal.",
      "enum": [
        "final",
        "draft",
        "hide"
      ],
      "type": "string"
    }
  },
  "description": "Structure of the payload of a Proposal Submission Action.",
  "maintainers": [
    {
      "name": "Catalyst Team",
      "url": "https://projectcatalyst.io/"
    }
  ],
  "properties": {
    "action": {
      "$ref": "#/definitions/action"
    }
  },
  "required": [
    "action"
  ],
  "title": "Proposal Submission Action Payload Schema",
  "x-changelog": {
    "2025-03-01": [
      "First Version Created."
    ]
  }
}
```
<!-- markdownlint-enable MD013 -->

## Signers

The following user roles may sign documents of this type:

* Proposer

New versions of this document may be published by:

* author
* collaborators

## Copyright

| Copyright | :copyright: 2024-2025 IOG Singapore, All Rights Reserved |
| --- | --- |
| License | This document is licensed under [CC-BY-4.0] |
| Created | 2024-12-27 |
| Modified | 2025-04-03 |
| Authors | Alex Pozhylenkov <alex.pozhylenkov@iohk.io> |
| | Steven Johnson <steven.johnson@iohk.io> |

[RFC9052-HeaderParameters]: https://www.rfc-editor.org/rfc/rfc8152#section-3.1
[application/json]: https://www.iana.org/assignments/media-types/application/json
[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
[RFC9562-V7]: https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-7
[RFC8259]: https://www.rfc-editor.org/rfc/rfc8259.html
