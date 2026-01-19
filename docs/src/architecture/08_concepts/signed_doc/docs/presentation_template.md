# Presentation Template

## Description

A Presentation Template defines how the data
captured by the *ANY* Set of Form Template or system dynamic data is to be displayed.

Multiple Presentation Templates will exist, and will be associated with multiple document types.
There is no strict 1:1 relationship between any document, its template and a presentation template.

The presentation templates are context sensitive, and each template defines the sources
of published document information they require.

Presentation Templates can reference any data contained
in the referenced Documents, as well as any documents linked by:

* [`ref`](../metadata.md#ref)
* [`reply`](../metadata.md#reply)
* [`parameters`](../metadata.md#parameters)

The presentation of the payload of all data when not capturing or displaying a
Form via its Form Template is controlled by one or more Presentation Template documents.

<!-- markdownlint-disable max-one-sentence-per-line -->

```graphviz dot presentation_template.dot.svg

{{ include_file('./../diagrams/presentation_template.dot', indent=4) }}
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
| Type | cb99b9bd-681a-49d8-9836-89107c02e8ef |
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
[`parameters`](../metadata.md#parameters) of the referencing document,
or a parent of those [`parameters`](../metadata.md#parameters).

For example, a linked reference to [Contest Parameters](contest_parameters.md) is transitively a reference to
the Parameters document it references, and each parameters document they reference
until the `Brand` parameters document is reached.

The use case here is for Templates.
The profile template, or proposal templates could be defined at any of these
levels, and as long as they all refer to the same chain of parameters in the
hierarchy they are all valid.

## Payload

The Presentation Template is defined by its schema.
See `[presentation_templates.md](TODO)`

### Schema

<!-- markdownlint-disable MD013 MD046 max-one-sentence-per-line -->
??? abstract "Schema: Payload [JSON][RFC8259] Schema"

    ```json
    {
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "maintainers": [
        {
          "name": "Catalyst Team",
          "url": "https://projectcatalyst.io/"
        }
      ],
      "title": "Presentation Template",
      "description": "Presentation Templates define how data extracted from Form Data is to be presented.\nThey provide a way to parameterize the contents of a UI in response to the changing\nneeds of the Forms themselves.",
      "$defs": {
        "cardDescription": {
          "description": "A long form description of the purpose of the card. Not used by the UI.",
          "type": "string"
        },
        "cardName": {
          "description": "A Card has to have one of the well known defined names.\nThese are the primary identifier which is used by the UI to determine\nwhere the UI will place the card.",
          "enum": [
            "draft-proposal-summary",
            "proposal-contest-summary"
          ],
          "type": "string"
        },
        "cardTemplate": {
          "contentType": "text/html; charset=utf-8; template=handlebars",
          "description": "HTML5 defined presentation layout for the card.\nThe data is templated with handlebars, and the data that can be inserted is\nderived from the `requiredDocumentTypes` and available system wide dynamic data.",
          "type": "string"
        },
        "cardTemplateCss": {
          "contentType": "text/css; charset=utf-8; template=handlebars",
          "description": "Optional styling that can be used by the HTML generated from the template for presentation.",
          "type": "string"
        },
        "cardTitle": {
          "description": "A title shown to the editor of the card.  Not used by the UI.",
          "type": "string"
        },
        "layoutParameters": {
          "description": "Parameters which help the front end layout the provided template. To be defined.",
          "type": "object"
        },
        "requiredDocumentTypes": {
          "description": "A list of the document types (UUIDs) the presentation template needs.",
          "items": {
            "format": "uuid",
            "type": "string"
          },
          "type": "array",
          "uniqueItems": true
        }
      },
      "type": "object",
      "properties": {
        "css": {
          "$ref": "#/$defs/cardTemplateCss"
        },
        "description": {
          "$ref": "#/$defs/cardDescription"
        },
        "layout": {
          "$ref": "#/$defs/layoutParameters"
        },
        "name": {
          "$ref": "#/$defs/cardName"
        },
        "requiredDocs": {
          "$ref": "#/$defs/requiredDocumentTypes"
        },
        "template": {
          "$ref": "#/$defs/cardTemplate"
        },
        "title": {
          "$ref": "#/$defs/cardTitle"
        }
      },
      "additionalProperties": false,
      "required": [
        "cardName",
        "requiredDocuments",
        "layoutParameters",
        "template"
      ]
    }
    ```
<!-- markdownlint-enable MD013 MD046 max-one-sentence-per-line -->

## Signers

The following Admin roles may sign documents of this type:

* Brand Admin
* Campaign Admin

Only the original author can update and sign a new version of documents.

## Copyright

| Copyright | :copyright: 2024-2026 IOG Singapore, All Rights Reserved |
| --- | --- |
| License | This document is licensed under [CC-BY-4.0] |
| Created | 2024-12-27 |
| Modified | 2026-01-15 |
| Authors | Alex Pozhylenkov <alex.pozhylenkov@iohk.io> |
| | Nathan Bogale <nathan.bogale@iohk.io> |
| | Neil McAuliffe <neil.mcauliffe@iohk.io> |
| | Steven Johnson <steven.johnson@iohk.io> |

### Changelog

#### 0.0.4 (2025-05-05)

* First Version.

#### 0.1.0 (2025-07-30)

* Updated to match Presentation Schema Definitions.

[RFC9052-HeaderParameters]: https://www.rfc-editor.org/rfc/rfc8152#section-3.1
[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
[RFC9562-V7]: https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-7
[RFC8259]: https://www.rfc-editor.org/rfc/rfc8259.html
