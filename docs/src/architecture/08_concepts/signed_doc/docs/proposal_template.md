# Proposal Template

## Description

## Proposal Template Document

A Proposal Template defines the allowed payload contents of a
linked proposal.

Proposals themselves are intentionally general, however they may be
linked to a brand/campaign or category via the template used by the proposal.

The payload of a proposal is controlled by its template.

```d2 layout="elk"
"Proposal Template": {
  shape: sql_table
  "content type": application/schema+json
  "type [0]": 0ce8ab38-9258-4fbc-a62e-7faa6e58318f
  "type [1]": 7808d2ba-d511-40af-84e8-c0d1625fdfdc
  "id": UUIDv7
  "ver": UUIDv7
  "template": Proposal Meta Template (Optional)
  "brand_id": Brand Parameters (Optional)
  "campaign_id": Campaign Parameters (Optional)
  "category_id": Category Parameters (Optional)

}

"Proposal Template"."template"->"Proposal Meta Template": Optional
"Proposal Template"."brand_id"->"Brand Parameters": Optional
"Proposal Template"."campaign_id"->"Campaign Parameters": Optional
"Proposal Template"."category_id"->"Category Parameters": Optional
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

* [content type](../spec.md#content-type) = `application/schema+json`
* [content-encoding](../spec.md#content-encoding) = `[br]`

## Metadata

### [`type`](../metadata.md#type)
<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Type](../metadata.md#document-type) |
| Type | `0ce8ab38-9258-4fbc-a62e-7faa6e58318f`,<br/>`7808d2ba-d511-40af-84e8-c0d1625fdfdc` |
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

### [`template`](../metadata.md#template)
<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | optional |
| Format | [Document Reference](../metadata.md#document-reference) |
| Valid References | [Proposal Meta Template](proposal_meta_template.md) |
<!-- markdownlint-enable MD033 -->
Reference to the template used to create and/or validate this document.

#### Validation

In addition to the validation performed for [Document Reference](../metadata.md#document-reference) type fields,
The document payload is not valid if it does not validate completely against the referenced template.

### [`brand_id`](../metadata.md#brand_id)
<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | optional |
| Format | [Document Reference](../metadata.md#document-reference) |
| Valid References | [Brand Parameters](brand_parameters.md) |
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

### [`campaign_id`](../metadata.md#campaign_id)
<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | optional |
| Format | [Document Reference](../metadata.md#document-reference) |
| Valid References | [Campaign Parameters](campaign_parameters.md) |
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

### [`category_id`](../metadata.md#category_id)
<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | optional |
| Format | [Document Reference](../metadata.md#document-reference) |
| Valid References | [Category Parameters](category_parameters.md) |
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

## Payload

[JSON Schema] document which defines the valid contents of a proposal document.

## Signers

The following admin roles may sign documents of this type:

* Brand Admin
* Campaign Admin

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
[JSON Schema]: https://json-schema.org/draft-07
[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
[RFC9562-V7]: https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-7
