# Voter Representative Delegation

## Description

## Voter Representative Delegation
A signed document that allows a 'Voter' to delegate their voting power to a 'Representative' for a specific category.

<!-- markdownlint-disable max-one-sentence-per-line -->

```graphviz dot voter_representative_delegation.dot.svg
{{ include_file('./../diagrams/voter_representative_delegation.dot', indent=4) }}
```

<!-- markdownlint-enable max-one-sentence-per-line -->

### Validation

* The signer MUST be a registered 'Voter'.
* The 'ref' metadata field MUST point to a valid 'Representative Category Profile'.
* The payload MUST be empty.

### Business Logic

#### Front End

* Allows a voter to select a Representative from a list of eligible candidates for a category.
* The voter signs this document to confirm their delegation choice.

#### Back End

* Verifies that the voter and Representative are valid and registered for the category.
* Records the delegation of voting power from the voter to the Representative.

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
| Type | `f1a2b3c4-3333-4abc-8def-2345678901cc` |
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

#### [`ver`](../metadata.md#ver) Validation

The document version must always be >= the document ID.

### [`ref`](../metadata.md#ref)

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Reference](../metadata.md#document-reference) |
| Valid References | [Representative Category Profile](representative_category_profile.md) |
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
| Valid References | [Category Parameters](category_parameters.md) |
<!-- markdownlint-enable MD033 -->
A reference to the Parameters Document this document lies under.

#### [`parameters`](../metadata.md#parameters) Validation

In addition to the validation performed for [Document Reference](../metadata.md#document-reference) type fields:

* Any linked referenced document that includes a [`parameters`](../metadata.md#parameters) metadata must match the
[`parameters`](../metadata.md#parameters) of the referencing document.

## Payload

A minimal payload indicating the intended status of the delegation.
  'active' creates or affirms the delegation.
  'revoked' withdraws the delegation.

### Schema

<!-- markdownlint-disable MD013 MD046 max-one-sentence-per-line -->
??? abstract

      A minimal payload indicating the intended status of the delegation.
      'active' creates or affirms the delegation.
      'revoked' withdraws the delegation.

    ```json
    {
      "$id": "https://raw.githubusercontent.com/input-output-hk/catalyst-libs/refs/heads/main/specs/signed_docs/docs/payload_schemas/voter_representative_delegation.schema.json",
      "$schema": "http://json-schema.org/draft-07/schema#",
      "additionalProperties": false,
      "description": "This payload is submitted when a voter delegates to a Representative, with the voter as the signer and the Representative\u2019s category profile as the referenced document; to revoke the delegation, the voter can publish again with the status set to revoked.",
      "maintainers": [
        {
          "name": "Catalyst Team",
          "url": "https://projectcatalyst.io/"
        }
      ],
      "properties": {
        "status": {
          "description": "The status of the delegation. 'active' signifies delegation (set by default), 'revoked' signifies withdrawal of delegation.",
          "enum": [
            "active",
            "revoked"
          ],
          "type": "string"
        }
      },
      "required": [
        "status"
      ],
      "title": "Voter Representative Delegation Payload Schema",
      "type": "object",
      "x-changelog": {
        "2025-06-19": [
          "First Version Created."
        ]
      }
    }
    ```

<!-- markdownlint-enable MD013 MD046 max-one-sentence-per-line -->

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
| Modified | 2025-06-19 |
| Authors | Alex Pozhylenkov <alex.pozhylenkov@iohk.io> |
| | Neil McAuliffe <neil.mcauliffe@iohk.io> |
| | Steven Johnson <steven.johnson@iohk.io> |

### Changelog

#### 0.01 (2025-06-19)

* First Published Version

[CBOR-TAG-42]: https://github.com/ipld/cid-cbor/
[RFC9052-HeaderParameters]: https://www.rfc-editor.org/rfc/rfc8152#section-3.1
[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
[IPFS-CID]: https://docs.ipfs.tech/concepts/content-addressing/#what-is-a-cid
[RFC9562-V7]: https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-7
