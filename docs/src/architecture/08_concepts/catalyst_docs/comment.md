---
Title: Catalyst Comment Document
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

## Comment Document

This is a document which provides a comment against a particular [Proposal Document].

### Specification

Catalyst Comment document is a [Catalyst Signed Document],
so its fully follows the structure of the [Catalyst Signed Document] specification.

#### Metadata Fields

A list of used [Catalyst Signed Document protected header fields](./../signed_doc/spec.md#signed-object-fields).

* [`type`](./../signed_doc/spec.md#type): `b679ded3-0e7c-41ba-89f8-da62a17898ea` [UUID] value.

  ```CDDL
  "type" => 37(h'b679ded30e7c41ba89f8da62a17898ea')
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

* [`ref`](./../signed_doc/meta.md#ref-document-reference).
  Reference to a related [Proposal Document],
  which [`type`](./../signed_doc/spec.md#type) must be equal to
  [proposal document `type`][Proposal Document] field value.

* [`template`](./../signed_doc/meta.md#ref-document-reference).
  A reference to the comment template document,
  which [`type`](./../signed_doc/spec.md#type) must be equal to
  [comment template `type`](#comment-template) field value.

* [`reply`](./../signed_doc/meta.md#reply-reply-reference) (optional).
  A reference to another comment document,
  where the comment is in reply to the referenced comment.
  The [`type`](./../signed_doc/spec.md#type) of the replied document
  must be equal to comment document `type` field value.
  Comments may only reply to a single other comment document.
  The referenced `comment` must be for the same proposal [`id`](./../signed_doc/spec.md#id),
  but can be for a different proposal [`ver`](./../signed_doc/spec.md#ver).

* [`section`](./../signed_doc/meta.md#section-section-reference) (optional).
  Used when the comment only applies to a specific section to the document being commented upon,
  and not the entire document.

#### Content format

TODO

## Comment Template

This document pr provides the template structure which a Comment must be formatted to, and validated against.

### Specification

Catalyst Comment Template document is a [Catalyst Signed Document],
so its fully follows the structure of the [Catalyst Signed Document] specification.

#### Metadata Fields

A list of used [Catalyst Signed Document protected header fields](./../signed_doc/spec.md#signed-object-fields).

* [`type`](./../signed_doc/spec.md#type): `0b8424d4-ebfd-46e3-9577-1775a69d290c` [UUID] value.

  ```CDDL
  "type" => 37(h'0b8424d4ebfd46e395771775a69d290c')
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
* [`category_id`](./../signed_doc/meta.md#category_id) (optional).
  A reference to the category document,
  which [`type`](./../signed_doc/spec.md#type) must be equal to
  `48c20109-362a-4d32-9bba-e0a9cf8b45be` value.

#### Fund 14 defined templates ids

* id: `0194d494-4402-7e0e-b8d6-171f8fea18b0`, ver: `0194d494-4402-7e0e-b8d6-171f8fea18b0`,
  category: `0194d490-30bf-7473-81c8-a0eaef369619`
* id: `0194d494-4402-7444-9058-9030815eb029`, ver: `0194d494-4402-7444-9058-9030815eb029`,
  category: `0194d490-30bf-7043-8c5c-f0e09f8a6d8c`
* id: `0194d494-4402-7351-b4f7-24938dc2c12e`, ver: `0194d494-4402-7351-b4f7-24938dc2c12e`,
  category: `0194d490-30bf-7e75-95c1-a6cf0e8086d9`
* id: `0194d494-4402-79ad-93ba-4d7a0b65d563`, ver: `0194d494-4402-79ad-93ba-4d7a0b65d563`,
  category: `0194d490-30bf-7703-a1c0-83a916b001e7`
* id: `0194d494-4402-7cee-a5a6-5739839b3b8a`, ver: `0194d494-4402-7cee-a5a6-5739839b3b8a`,
  category: `0194d490-30bf-79d1-9a0f-84943123ef38`
* id: `0194d494-4402-7aee-8b24-b5300c976846`, ver: `0194d494-4402-7aee-8b24-b5300c976846`,
  category: `0194d490-30bf-706d-91c6-0d4707f74cdf`
* id: `0194d494-4402-7d75-be7f-1c4f3471a53c`, ver: `0194d494-4402-7d75-be7f-1c4f3471a53c`,
  category: `0194d490-30bf-759e-b729-304306fbaa5e`
* id: `0194d494-4402-7a2c-8971-1b4c255c826d`, ver: `0194d494-4402-7a2c-8971-1b4c255c826d`,
  category: `0194d490-30bf-7e27-b5fd-de3133b54bf6`
* id: `0194d494-4402-7074-86ac-3efd097ba9b0`, ver: `0194d494-4402-7074-86ac-3efd097ba9b0`,
  category: `0194d490-30bf-7f9e-8a5d-91fb67c078f2`
* id: `0194d494-4402-7202-8ebb-8c4c47c286d8`, ver: `0194d494-4402-7202-8ebb-8c4c47c286d8`,
  category: `0194d490-30bf-7676-9658-36c0b67e656e`
* id: `0194d494-4402-7fb5-b680-c23fe4beb088`, ver: `0194d494-4402-7fb5-b680-c23fe4beb088`,
  category: `0194d490-30bf-7978-b031-7aa2ccc5e3fd`
* id: `0194d494-4402-7aa5-9dbc-5fe886e60ebc`, ver: `0194d494-4402-7aa5-9dbc-5fe886e60ebc`,
  category: `0194d490-30bf-7d34-bba9-8498094bd627`

#### Content format

TODO

## Copyright

This document is licensed under [CC-BY-4.0](https://creativecommons.org/licenses/by/4.0/legalcode).

[Catalyst Signed Document]: ./../signed_doc/spec.md
[Catalyst Signed Document content]: ./../signed_doc/spec.md#signed-object-content
[Proposal Document]: ./proposal.md
[Brotli]: https://datatracker.ietf.org/doc/html/rfc7932
[JSON]: https://datatracker.ietf.org/doc/html/rfc7159
[UUID]: https://www.rfc-editor.org/rfc/rfc9562.html
