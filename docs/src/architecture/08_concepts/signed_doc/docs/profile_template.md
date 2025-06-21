# Profile Template

## Description

## Profile Template Document

Defines the allowed payload contents and constraints for a generic user profile.

<!-- markdownlint-disable max-one-sentence-per-line -->

```graphviz dot profile_template.dot.svg
{{ include_file('./../diagrams/profile_template.dot', indent=4) }}
```

<!-- markdownlint-enable max-one-sentence-per-line -->

### Validation

* The signer MUST be a registered 'Admin'.
* The payload MUST be a valid [JSON schema].
* The schema SHOULD define a minimal set of profile fields (e.g., name, bio).

### Business Logic

#### Front End



#### Back End

* Validate and store the [JSON schema] that defines the structure for all 'Profile' documents.

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
| Type | `0ce8ab38-9258-4fbc-a62e-7faa6e58318f`,<br/>`1b70f611-518d-479e-be73-11b5e9cb68a5` |
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

[JSON Schema] document which defines the valid contents of a profile document.

### Schema

<!-- markdownlint-disable MD013 MD046 max-one-sentence-per-line -->
??? abstract

    [JSON Schema] document which defines the valid contents of a profile document.

    ```json
    {
      "$id": "https://raw.githubusercontent.com/input-output-hk/catalyst-libs/refs/heads/main/specs/signed_docs/docs/payload_schemas/profile_template.schema.json",
      "$schema": "http://json-schema.org/draft-07/schema#",
      "additionalProperties": false,
      "description": "Schema for a profile document template for any Catalyst actor.",
      "maintainers": [
        {
          "name": "Catalyst Team",
          "url": "https://projectcatalyst.io/"
        }
      ],
      "properties": {
        "bio": {
          "type": "string"
        },
        "name": {
          "type": "string"
        }
      },
      "required": [
        "name",
        "bio"
      ],
      "title": "Profile Template Payload Schema",
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
