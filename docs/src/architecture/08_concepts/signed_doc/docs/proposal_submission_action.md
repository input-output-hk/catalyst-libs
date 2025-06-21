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

<!-- markdownlint-disable max-one-sentence-per-line -->

```graphviz dot proposal_submission_action.dot.svg
{{ include_file('./../diagrams/proposal_submission_action.dot', indent=4) }}
```

<!-- markdownlint-enable max-one-sentence-per-line -->

### Validation

No validation is required beyond as defined by:

* [metadata](#metadata)
* [payload](#payload)
* [signers](#signers)

### Business Logic

#### Front End

A proposal with [`collaborators`](../metadata.md#collaborators) will not be shown as having a confirmed collaborator,
unless there exists a `draft` or `final` proposal submission from that collaborator.

Any document that lists a collaborator should be highlighted to that collaborator so
they can take appropriate action, such as:

* Confirm they are a collaborator by submitting this document as `draft`
* Agree to being a collaborator on the final submission by submitting this document as `final`
* Hide themselves from the collaborators list but do not remove themselves by submitting `hide`
* Remove themselves permanently as a collaborator by publishing a new version with them removed.

To eliminate the necessity for collaborators to accept collaboration on every version,
they will be considered as agreeing to be a collaborator on any version of the document
that lists them, if their latest submission is `draft` or `final`.

If their latest submission on a document is `hide` they should be considered to not
have agreed to be a collaborator.

*NOTE* `final` status ONLY applies to the exactly referenced document and version.

#### Back End

A Submitted proposal with collaborators *MUST* have
a `final` submission by *ALL* listed [`collaborators`](../metadata.md#collaborators).
If any `collaborator` has not submitted a `final` submission by the deadline, then the proposal
is not considered `final` and will not be considered in the category it was being submitted to.

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
| Type | `5e60e623-ad02-4a1b-a1ac-406db978ee48`,<br/>`7808d2ba-d511-40af-84e8-c0d1625fdfdc`,<br/>`78927329-cfd9-4ea1-9c71-0e019b126a65` |
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
<!-- markdownlint-enable MD033 -->
A reference to the Parameters Document this document lies under.

#### [`parameters`](../metadata.md#parameters) Validation

In addition to the validation performed for [Document Reference](../metadata.md#document-reference) type fields:

* Any linked referenced document that includes a [`parameters`](../metadata.md#parameters) metadata must match the
[`parameters`](../metadata.md#parameters) of the referencing document.
* The Document referenced by [`ref`](../metadata.md#ref)
  * MUST contain [`parameters`](../metadata.md#parameters) metadata; AND
  * MUST match the referencing documents [`parameters`](../metadata.md#parameters) value.

## Payload

The kind of action is controlled by this payload.
The Payload is a [JSON][RFC8259] Document, and must conform to this schema.

States:

* `final` : All collaborators must publish a `final` status for the proposal to be `final`.
* `draft` : Reverses the previous `final` state for a signer and accepts collaborator status to a document.
* `hide`  : Requests the proposal be hidden (not final, but a hidden draft).
       `hide` is only actioned if sent by the author,
      for a collaborator it identified that they do not wish to be listed as a `collaborator`.

### Schema

<!-- markdownlint-disable MD013 MD046 max-one-sentence-per-line -->
??? abstract

    The kind of action is controlled by this payload.
    The Payload is a [JSON][RFC8259] Document, and must conform to this schema.

    States:

    * `final` : All collaborators must publish a `final` status for the proposal to be `final`.
    * `draft` : Reverses the previous `final` state for a signer and accepts collaborator status to a document.
    * `hide`  : Requests the proposal be hidden (not final, but a hidden draft).
           `hide` is only actioned if sent by the author,
          for a collaborator it identified that they do not wish to be listed as a `collaborator`.

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
      "type": "object",
      "x-changelog": {
        "2025-03-01": [
          "First Version Created."
        ]
      }
    }
    ```

<!-- markdownlint-enable MD013 MD046 max-one-sentence-per-line -->

### Examples
<!-- markdownlint-disable MD013 MD046 max-one-sentence-per-line -->
??? example "Example: Final Proposal Submission"

    This document indicates the linked proposal is final and requested to proceed for further consideration.

    ```json
    {
      "action": "final"
    }
    ```

<!-- markdownlint-enable MD013 MD046 max-one-sentence-per-line -->
<!-- markdownlint-disable MD013 MD046 max-one-sentence-per-line -->
??? example "Example: Draft Proposal Submission"

    This document indicates the linked proposal is no longer final and should not proceed for further consideration.
    It is also used by collaborators to accept that they are a collaborator on a document.

    ```json
    {
      "action": "draft"
    }
    ```

<!-- markdownlint-enable MD013 MD046 max-one-sentence-per-line -->
<!-- markdownlint-disable MD013 MD046 max-one-sentence-per-line -->
??? example "Example: Hidden Proposal Submission"

    If submitted by the proposal author the document is hidden, it is still public but not shown as
    a proposal being drafted.
    If submitted by a collaborator, that collaborator is declaring they do not wish to be listed as
    a collaborator on the proposal.

    ```json
    {
      "action": "hide"
    }
    ```

<!-- markdownlint-enable MD013 MD046 max-one-sentence-per-line -->

## Signers

The following User roles may sign documents of this type:

* Proposer

New versions of this document may be published by:

* author
* collaborators

## Copyright

| Copyright | :copyright: 2024-2025 IOG Singapore, All Rights Reserved |
| --- | --- |
| License | This document is licensed under [CC-BY-4.0] |
| Created | 2024-12-27 |
| Modified | 2025-05-30 |
| Authors | Alex Pozhylenkov <alex.pozhylenkov@iohk.io> |
| | Steven Johnson <steven.johnson@iohk.io> |

### Changelog

#### 0.01 (2025-04-04)

* First Published Version

#### 0.03 (2025-05-05)

* Use generalized parameters.

[CBOR-TAG-42]: https://github.com/ipld/cid-cbor/
[RFC9052-HeaderParameters]: https://www.rfc-editor.org/rfc/rfc8152#section-3.1
[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
[IPFS-CID]: https://docs.ipfs.tech/concepts/content-addressing/#what-is-a-cid
[RFC9562-V7]: https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-7
[RFC8259]: https://www.rfc-editor.org/rfc/rfc8259.html
