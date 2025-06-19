# Representative Category Profile

## Description

## Representative Category Profile Document

A Representative Category Profile is created to opt in as a Representative for a specific campaign category, the user must have registered as a Representative.
The presence of this docuemnt signifies the user's intent to participate in that category as a Representative.

The document's structure is defined by the associated Representative_Category_Profile_Template, which allows an Admin to specify category-specific requirements.

The payload must contain a 'status' field to indicate if the Representative is active or has revoked their participation.

<!-- markdownlint-disable max-one-sentence-per-line -->

```graphviz dot representative_category_profile.dot.svg
{{ include_file('./../diagrams/representative_category_profile.dot', indent=4) }}
```

<!-- markdownlint-enable max-one-sentence-per-line -->

### Validation

  * The signer MUST be a registered 'Representative'.
 * The 'ref' metadata field MUST point to a valid 'Representative Profile' document.
    * The 'parameters' metadata field MUST point to a valid 'Category Parameters' document.
   * The 'template' metadata field MUST point to a valid 'Representative Category Profile Template' document.
   * The payload MUST be valid against the [JSON schema] defined in the referenced template.

### Business Logic

#### Front End

 * Allows a Representative to create or update their profile for a category.
    * The Representative sets their status to 'active' to be discoverable for delegation.
 * The Representative can set their status to 'revoked' to signal they are no longer participating in the category,
   without having to revoke the document.

#### Back End

   * The backend MUST verify the signer is a 'Representative' and that all referenced documents exist.
  * The system will only consider Representatives with an 'active' status as eligible for delegation.

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
| Type | `f1a2b3c4-1111-4abc-8def-2345678901aa` |
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
| Valid References | [Representative Profile](representative_profile.md) |
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

### [`template`](../metadata.md#template)

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Reference](../metadata.md#document-reference) |
| Valid References | [Representative Category Profile Template](representative_category_profile_template.md) |
<!-- markdownlint-enable MD033 -->
Reference to the template used to create and/or validate this document.

#### [`template`](../metadata.md#template) Validation

In addition to the validation performed for [Document Reference](../metadata.md#document-reference) type fields,
The document payload is not valid if it does not validate completely against the referenced template.

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

The Representative's profile data for a specific category.
    Its structure is defined by the referenced template document.
 It MUST contain a 'status' field ('active' or 'revoked') to manage the Representative's participation.

## Signers

The following user roles may sign documents of this type:

* Representative

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
[JSON Schema]: https://json-schema.org/draft-07
[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
[IPFS-CID]: https://docs.ipfs.tech/concepts/content-addressing/#what-is-a-cid
[RFC9562-V7]: https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-7
