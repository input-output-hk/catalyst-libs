---
Title: Catalyst Review Document
Category: Catalyst
Status: Proposed
Authors:
    - Steven Johnson <steven.johnson@iohk.io>
Implementors:
    - Catalyst Fund 14
Discussions: []
Created: 2024-12-29
License: CC-BY-4.0
---

## Abstract

## Review Document

TODO

### Specification

Catalyst Review document is a [Catalyst Signed Document],
so its fully follows the structure of the [Catalyst Signed Document] specification.

#### Metadata Fields

A list of used [Catalyst Signed Document protected header fields](./../signed_doc/spec.md#signed-object-fields).

* [`type`](./../signed_doc/spec.md#type): `e4caf5f0-098b-45fd-94f3-0702a4573db5` [UUID] value.

  ```CDDL
  "type" => 37(h'e4caf5f0098b45fd94f30702a4573db5')
  ```

* [`content type`](./../signed_doc/spec.md#content-type): `application/json`.
  [Catalyst Signed Document content] must be in [JSON] format.

  ```CDDL
  3 => 30
  ```

* [`content encoding`](./../signed_doc/spec.md#content-encoding-optional):
  [Catalyst Signed Document content] must be [Brotli] compressed.

  ```CDDL
  "content-encoding" => "br"
  ```

* [`template`](./../signed_doc/metadata.md#ref-document-reference).
  A reference to the review template document,
  which [`type`](./../signed_doc/spec.md#type) must be equal to
  [review template `type`](#review-template) field value.

#### Content format

TODO

## Review Template

TODO

### Specification

Catalyst Review Template document is a [Catalyst Signed Document],
so its fully follows the structure of the [Catalyst Signed Document] specification.

#### Metadata Fields

A list of used [Catalyst Signed Document protected header fields](./../signed_doc/spec.md#signed-object-fields).

* [`type`](./../signed_doc/spec.md#type): `ebe5d0bf-5d86-4577-af4d-008fddbe2edc` [UUID] value.

  ```CDDL
  "type" => 37(h'ebe5d0bf5d864577af4d008fddbe2edc')
  ```

* [`content type`](./../signed_doc/spec.md#content-type): `application/json`.
  [Catalyst Signed Document content] must be in [JSON] format.

  ```CDDL
  3 => 30
  ```

* [`content encoding`](./../signed_doc/spec.md#content-encoding-optional):
  [Catalyst Signed Document content] must be [Brotli] compressed.

  ```CDDL
  "content-encoding" => "br"
  ```

#### Content format

TODO

## Copyright

This document is licensed under [CC-BY-4.0](https://creativecommons.org/licenses/by/4.0/legalcode).

[Catalyst Signed Document]: ./../signed_doc/spec.md
[Catalyst Signed Document content]: ./../signed_doc/spec.md#signed-object-content
[Brotli]: https://datatracker.ietf.org/doc/html/rfc7932
[JSON]: https://datatracker.ietf.org/doc/html/rfc7159
[UUID]: https://www.rfc-editor.org/rfc/rfc9562.html
