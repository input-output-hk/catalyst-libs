# Proposal Comment Presentation Template

## Description

A Proposal Comment Presentation Template defines how the data
captured by the Proposal Comment Form Template is to be displayed.

Multiple Proposal Comment Presentation Templates can exist for the
same Proposal Comment Form Template.
Each can be used to display the form data under different
circumstances.

Proposal Comment Presentation Templates can reference any data contained
in the Proposal Comment Document, as well as any documents linked by:

* [`ref`](../metadata.md#ref)
* [`reply`](../metadata.md#reply)
* [`parameters`](../metadata.md#parameters)

The presentation of the payload of a Proposal Comment is controlled by
its Proposal Comment Presentation Template/s.

<!-- markdownlint-disable max-one-sentence-per-line -->

```graphviz dot proposal_comment_presentation_template.dot.svg
{{ include_file('./../diagrams/proposal_comment_presentation_template.dot', indent=4) }}
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
| Type | `cb99b9bd-681a-49d8-9836-89107c02e8ef`,<br/>`b679ded3-0e7c-41ba-89f8-da62a17898ea`,<br/>`7808d2ba-d511-40af-84e8-c0d1625fdfdc` |
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

### [`parameters`](../metadata.md#parameters)

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Reference](../metadata.md#document-reference) |
| Valid References | [Brand Parameters](brand_parameters.md) |
|  | [Campaign Parameters](campaign_parameters.md) |
|  | [Category Parameters](category_parameters.md) |
<!-- markdownlint-enable MD033 -->
A reference to the Parameters Document this document lies under.

#### [`parameters`](../metadata.md#parameters) Validation

In addition to the validation performed for [Document Reference](../metadata.md#document-reference) type fields:

* Any linked referenced document that includes a [`parameters`](../metadata.md#parameters) metadata must match the
[`parameters`](../metadata.md#parameters) of the referencing document.

## Payload

TBD.
But roughly, will be:

1. A way to identify where the presentation template is intended to be used.
2. Optional [CSS] to control the presentation.
3. A [Handlebars] templated [HTML][HTML5] or [Markdown][CommonMark] file data which defines the presentation.

## Signers

The following Admin roles may sign documents of this type:

* Brand Admin
* Campaign Admin

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

#### 0.04 (2025-05-05)

* First Version.

[RFC9052-HeaderParameters]: https://www.rfc-editor.org/rfc/rfc8152#section-3.1
[Handlebars]: https://handlebarsjs.com/
[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
[CommonMark]: https://spec.commonmark.org/0.31.2/
[RFC9562-V7]: https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-7
[HTML5]: https://html.spec.whatwg.org/multipage/syntax.html#syntax
[CSS]: https://www.w3.org/Style/CSS/
