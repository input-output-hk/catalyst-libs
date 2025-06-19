# Representative Category Profile Template

## Description

## Representative Category Profile Template

Defines the [JSON schema] for a 'Representative Category Profile'.
This allows an 'Admin' to specify different profile requirements for each category.

<!-- markdownlint-disable max-one-sentence-per-line -->

```graphviz dot representative_category_profile_template.dot.svg
{{ include_file('./../diagrams/representative_category_profile_template.dot', indent=4) }}
```

<!-- markdownlint-enable max-one-sentence-per-line -->

### Validation

* The signer MUST be a registered 'Admin'.
* The payload MUST be a valid [JSON schema].
* The schema MUST include a 'status' field.

### Business Logic

#### Front End



#### Back End

* Validate and store the [JSON schema] that defines the structure for 'Representative Category Profile' documents.

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
| Type | `0ce8ab38-9258-4fbc-a62e-7faa6e58318f`,<br/>`f1a2b3c4-1111-4abc-8def-2345678901aa` |
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

## Payload

[JSON Schema] document which defines the valid contents of a Representative Category Profile document.
The schema MUST include a 'status' field to indicate if the Representative is active or withdrawn from the category.

### Schema

<!-- markdownlint-disable MD013 MD046 max-one-sentence-per-line -->
??? abstract

    [JSON Schema] document which defines the valid contents of a Representative Category Profile document.
    The schema MUST include a 'status' field to indicate if the Representative is active or withdrawn from the category.

    ```json
    {
      "$id": "https://raw.githubusercontent.com/input-output-hk/catalyst-libs/refs/heads/main/specs/signed_docs/docs/payload_schemas/representative_category_profile_template.schema.json",
      "$schema": "http://json-schema.org/draft-07/schema#",
      "additionalProperties": true,
      "definitions": {
        "status": {
          "description": "The Representative's status in this category. 'active' means they are participating, 'revoked' means they have withdrawn.",
          "enum": [
            "active",
            "revoked"
          ],
          "type": "string"
        }
      },
      "description": "This schema is defined by an Admin to specify the required properties for a user opting in as a Representative within a specific campaign category. It outlines the structure of a Representative's profile for that category and supports the addition of custom properties as needed by the Admin. The status field is mandatory and cannot be removed or modified by the Admin.",
      "maintainers": [
        {
          "name": "Catalyst Team",
          "url": "https://projectcatalyst.io/"
        }
      ],
      "properties": {
        "status": {
          "$ref": "#/definitions/status"
        }
      },
      "required": [
        "status"
      ],
      "title": "Representative Category Profile Template Payload Schema",
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

The following admin roles may sign documents of this type:

* Brand Admin
* Campaign Admin

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

[RFC9052-HeaderParameters]: https://www.rfc-editor.org/rfc/rfc8152#section-3.1
[JSON Schema]: https://json-schema.org/draft-07
[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
[RFC9562-V7]: https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-7
