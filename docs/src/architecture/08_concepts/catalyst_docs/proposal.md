---
Title: Catalyst Proposal Document
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

## Proposal Document

This is a document, formatted against the referenced proposal template, which defines a proposal which may be submitted
for consideration under one or more brand campaign categories.

The brand, campaign and category are not part of the document because the document can exist outside this boundary.
They are defined when a specific document is submitted for consideration.

### Specification

Catalyst Proposal document is a [Catalyst Signed Document],
so its fully follows the structure of the [Catalyst Signed Document] specification.

#### Metadata Fields

A list of used [Catalyst Signed Document protected header fields](./../signed_doc/spec.md#signed-object-fields).

* [`type`](./../signed_doc/spec.md#type): `7808d2ba-d511-40af-84e8-c0d1625fdfdc` [UUID] value.

  ```CDDL
  "type" => 37(h'7808d2bad51140af84e8c0d1625fdfdc')
  ```

* [`content type`](./../signed_doc/spec.md#content-type): `application/json`.
  [Catalyst Signed Document content] must be in [JSON] format.

  ```CDDL
  3 => 30
  ```

* [`content encoding`](./../signed_doc/spec.md#content-encoding-optional):
  [Catalyst Signed Document content] must be [Brotli] compressed.

  ```CDDL
  "content-type" => "br"
  ```

* [`ref`](./../signed_doc/meta.md#ref-document-reference) (optional).
* [`template`](./../signed_doc/meta.md#ref-document-reference) must be equal to `0ce8ab38-9258-4fbc-a62e-7faa6e58318f` value,
  [proposal template type](#proposal-template).

  ```CDDL
  "template" => 37(h'0ce8ab3892584fbca62e7faa6e58318f')
  ```

* [`reply`](./../signed_doc/meta.md#reply-reply-reference) (optional).
* [`section`](./../signed_doc/meta.md#section-section-reference) (optional).
* [`collabs`](./../signed_doc/meta.md#collabs-authorized-collaborators) (optional).

## Proposal Template

This document provides the template structure which a Proposal must be formatted to, and validated against.

### Specification

Catalyst Proposal Template document is a [Catalyst Signed Document],
so its fully follows the structure of the [Catalyst Signed Document] specification.

#### Metadata Fields

A list of used [Catalyst Signed Document protected header fields](./../signed_doc/spec.md#signed-object-fields).

* [`type`](./../signed_doc/spec.md#type): `0ce8ab38-9258-4fbc-a62e-7faa6e58318f` [UUID] value.

  ```CDDL
  "type" => 37(h'0ce8ab3892584fbca62e7faa6e58318f')
  ```

* [`content type`](./../signed_doc/spec.md#content-type): `application/json`.
  [Catalyst Signed Document content] must be in [JSON] format.

  ```CDDL
  3 => 30
  ```

* [`content encoding`](./../signed_doc/spec.md#content-encoding-optional):
  [Catalyst Signed Document content] must be [Brotli] compressed.

  ```CDDL
  "content-type" => "br"
  ```

* [`ref`](./../signed_doc/meta.md#ref-document-reference) (optional)
* [`reply`](./../signed_doc/meta.md#reply-reply-reference) (optional)
* [`section`](./../signed_doc/meta.md#section-section-reference) (optional)
* [`collabs`](./../signed_doc/meta.md#collabs-authorized-collaborators) (optional)

## Copyright

This document is licensed under [CC-BY-4.0](https://creativecommons.org/licenses/by/4.0/legalcode).

[Catalyst Signed Document]: ./../signed_doc/spec.md
[Catalyst Signed Document content]: ./../signed_doc/spec.md#signed-object-content
[Brotli]: https://datatracker.ietf.org/doc/html/rfc7932
[JSON]: https://datatracker.ietf.org/doc/html/rfc7159
[UUID]: https://www.rfc-editor.org/rfc/rfc9562.html
