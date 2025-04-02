# Proposal Comment Meta Template

## Description

## Proposal Comment Meta Template Document

A Proposal Comment Meta Template is used to enforce functional requirements
are met in any Proposal Comment Template.

The payload of a proposal comment template is controlled by its meta template.

### Validation

TODO

### Business Logic

#### Front End

TODO

#### Back End

TODO

## [COSE Header Parameters][RFC9052-HeaderParameters]

* [content type](../spec.md#content-type) = `application/schema+json`
* [content-encoding](../spec.md#content-encoding) = `[br]`

## Metadata

### `type`
<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Type](../metadata.md#document-type) |
| Type | `0ce8ab38-9258-4fbc-a62e-7faa6e58318f`,<br/>`0ce8ab38-9258-4fbc-a62e-7faa6e58318f`,<br/>`b679ded3-0e7c-41ba-89f8-da62a17898ea`,<br/>`7808d2ba-d511-40af-84e8-c0d1625fdfdc` |
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

### `category_id`
<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | optional |
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

[JSON Schema] document which ensures the minimum required functional requirements
of the Proposal Comment Template are met.

This ensures that payloads can be reliably interpreted by business logic processes,
while allowing for flexibility to capture extended information.

**Must be a valid [JSON Schema] Draft 7 document.**

## Signers

The following admin roles may sign documents of this type:

* Root Admin
* Brand Admin

New versions of this document may be published by:

* author

## Copyright

| Copyright | :copyright: 2024-2025 IOG Singapore, All Rights Reserved |
| --- | --- |
| License | This document is licensed under [CC-BY-4.0] |
| Created | 2024-12-27 |
| Modified | 2025-04-02 |
| Authors | Alex Pozhylenkov <alex.pozhylenkov@iohk.io> |
| | Steven Johnson <steven.johnson@iohk.io> |

[RFC9052-HeaderParameters]: https://www.rfc-editor.org/rfc/rfc8152#section-3.1
[JSON Schema]: https://json-schema.org/draft-07
[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
[RFC9562-V7]: https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-7
