# Category Parameters Form Template

## Description

A Category Parameters Form Template defines both:

* The data that is entered in the Form.
* Formatting hints for the collection of the data in a form.

A Category Parameters Form Template is a [JSON Schema][JSON Schema-2020-12] Document.

Category Parameters entry *SHOULD* use the hints when collecting
data defined by the Category Parameters Form Template to provide a
consistent user interface.
It *CAN* also use those hints when re-displaying the full forms data.

Alternatively a Category Parameters Presentation Template can be used to
format the Category Parameters data for presentation.

The Category Parameters Document is intentionally general,
however it may be linked to a brand/campaign or category
via the template used by the Category Parameters.

The payload of a Category Parameters is controlled by its template.

<!-- markdownlint-disable max-one-sentence-per-line -->

```graphviz dot category_parameters_form_template.dot.svg
{{ include_file('./../diagrams/category_parameters_form_template.dot', indent=4) }}
```

<!-- markdownlint-enable max-one-sentence-per-line -->

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
| Type | `818938c3-3139-4daa-afe6-974c78488e95`,<br/>`60185874-7e13-407c-a06c-238ffe637ae6`,<br/>`0ce8ab38-9258-4fbc-a62e-7faa6e58318f` |
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

## Payload

[JSON Schema][JSON Schema-2020-12] document which defines the valid contents and
formatting hints for the collection of data for a
Category Parameters Document.

## Signers

The following Admin roles may sign documents of this type:

* Brand Admin

New versions of this document may be published by:

* author

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

#### 0.04 (2025-05-05)

* Generalize the Form Template definitions.

[RFC9052-HeaderParameters]: https://www.rfc-editor.org/rfc/rfc8152#section-3.1
[JSON Schema-2020-12]: https://json-schema.org/draft/2020-12
[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
[RFC9562-V7]: https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-7
