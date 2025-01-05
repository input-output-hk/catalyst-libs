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

* [`content encoding`](./../signed_doc/spec.md#content-encoding-optional): [Catalyst Signed Document content] must be [Brotli] compressed.

  ```CDDL
  "content-type" => "br"
  ```

* [`ref`](./../signed_doc/meta.md#ref-document-reference). Reference to a related [Proposal Document].
* [`template`](./../signed_doc/meta.md#ref-document-reference) must be equal to `0b8424d4-ebfd-46e3-9577-1775a69d290c` value, [comment template type](#comment-template).

  ```CDDL
  "template" => 37(h'0b8424d4ebfd46e395771775a69d290c')
  ```

* [`reply`](./../signed_doc/meta.md#reply-reply-reference) (optional).
  A reference to another comment,
  where the comment is in reply to the referenced comment.
  Comments may only reply to a single other comment document.
  The referenced `comment` must be for the same proposal [`id`](./../signed_doc/spec.md#id),
  but can be for a different proposal [`ver`](./../signed_doc/spec.md#ver).

* [`section`](./../signed_doc/meta.md#section-section-reference) (optional).
  Used when the comment only applies to a specific section to the document being commented upon,
  and not the entire document.

* [`collabs`](./../signed_doc/meta.md#collabs-authorized-collaborators) (optional).

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

* [`content encoding`](./../signed_doc/spec.md#content-encoding-optional): [Catalyst Signed Document content] must be [Brotli] compressed.

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
[Proposal Document]: ./proposal.md
[Brotli]: https://datatracker.ietf.org/doc/html/rfc7932
[JSON]: https://datatracker.ietf.org/doc/html/rfc7159
[UUID]: https://www.rfc-editor.org/rfc/rfc9562.html
