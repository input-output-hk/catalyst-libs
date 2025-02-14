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
  "content-encoding" => "br"
  ```

* [`template`](./../signed_doc/meta.md#ref-document-reference).
  A reference to the proposal template document,
  which [`type`](./../signed_doc/spec.md#type) must be equal to
  [proposal template `type`](#proposal-template) field value.

* [`category_id`](./../signed_doc/meta.md#category_id) (optional).
  A reference to the category document,
  which [`type`](./../signed_doc/spec.md#type) must be equal to
  `48c20109-362a-4d32-9bba-e0a9cf8b45be` value.

#### Fund 14 defined category ids

- id: `0194d490-30bf-7473-81c8-a0eaef369619`
- id: `0194d490-30bf-7043-8c5c-f0e09f8a6d8c`
- id: `0194d490-30bf-7e75-95c1-a6cf0e8086d9`
- id: `0194d490-30bf-7703-a1c0-83a916b001e7`
- id: `0194d490-30bf-79d1-9a0f-84943123ef38`
- id: `0194d490-30bf-706d-91c6-0d4707f74cdf`
- id: `0194d490-30bf-759e-b729-304306fbaa5e`
- id: `0194d490-30bf-7e27-b5fd-de3133b54bf6`
- id: `0194d490-30bf-7f9e-8a5d-91fb67c078f2`
- id: `0194d490-30bf-7676-9658-36c0b67e656e`
- id: `0194d490-30bf-7978-b031-7aa2ccc5e3fd`
- id: `0194d490-30bf-7d34-bba9-8498094bd627`

#### Content format

TODO

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
  "content-encoding" => "br"
  ```

#### Fund 14 defined templates ids

- id: `0194d492-1daa-75b5-b4a4-5cf331cd8d1a`, ver: `0194d492-1daa-75b5-b4a4-5cf331cd8d1a`
- id: `0194d492-1daa-7371-8bd3-c15811b2b063`, ver: `0194d492-1daa-7371-8bd3-c15811b2b063`
- id: `0194d492-1daa-79c7-a222-2c3b581443a8`, ver: `0194d492-1daa-79c7-a222-2c3b581443a8`
- id: `0194d492-1daa-716f-a04e-f422f08a99dc`, ver: `0194d492-1daa-716f-a04e-f422f08a99dc`
- id: `0194d492-1daa-78fc-818a-bf20fc3e9b87`, ver: `0194d492-1daa-78fc-818a-bf20fc3e9b87`
- id: `0194d492-1daa-7d98-a3aa-c57d99121f78`, ver: `0194d492-1daa-7d98-a3aa-c57d99121f78`
- id: `0194d492-1daa-77be-a1a5-c238fe25fe4f`, ver: `0194d492-1daa-77be-a1a5-c238fe25fe4f`
- id: `0194d492-1daa-7254-a512-30a4cdecfb90`, ver: `0194d492-1daa-7254-a512-30a4cdecfb90`
- id: `0194d492-1daa-7de9-b535-1a0b0474ed4e`, ver: `0194d492-1daa-7de9-b535-1a0b0474ed4e`
- id: `0194d492-1daa-7fce-84ee-b872a4661075`, ver: `0194d492-1daa-7fce-84ee-b872a4661075`
- id: `0194d492-1daa-7878-9bcc-2c79fef0fc13`, ver: `0194d492-1daa-7878-9bcc-2c79fef0fc13`
- id: `0194d492-1daa-722f-94f4-687f2c068a5d`, ver: `0194d492-1daa-722f-94f4-687f2c068a5d`

#### Content format

TODO

## Copyright

This document is licensed under [CC-BY-4.0](https://creativecommons.org/licenses/by/4.0/legalcode).

[Catalyst Signed Document]: ./../signed_doc/spec.md
[Catalyst Signed Document content]: ./../signed_doc/spec.md#signed-object-content
[Brotli]: https://datatracker.ietf.org/doc/html/rfc7932
[JSON]: https://datatracker.ietf.org/doc/html/rfc7159
[UUID]: https://www.rfc-editor.org/rfc/rfc9562.html
