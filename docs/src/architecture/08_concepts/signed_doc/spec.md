---
Title: Catalyst Signed Document
Category: Catalyst
Status: Proposed
Authors:
    - Steven Johnson <steven.johnson@iohk.io>
    - Alex Pozhylenkov <alex.pozhylenkov@iohk.io>
Implementors:
    - Catalyst Fund 14
Discussions: []
Created: 2024-12-27
License: CC-BY-4.0
---

* [Abstract](#abstract)
* [Motivation: why is this CIP necessary?](#motivation-why-is-this-cip-necessary)
* [Specification](#specification)
  * [Catalyst Signed Document metadata fields](#catalyst-signed-document-metadata-fields)
    * [`type`](#type)
    * [`id`](#id)
    * [`ver`](#ver)
    * [`alg`](#alg)
    * [`content type`](#content-type)
    * [`content encoding` (optional)](#content-encoding-optional)
  * [Catalyst Signed Document content](#catalyst-signed-document-content)
  * [COSE signature protected header](#cose-signature-protected-header)
* [Copyright](#copyright)

## Abstract

Project Catalyst both produces and consumes a lot of different data objects,
in different places of the system.
To ensure the data object is authoritative, it must be signed.
Id addition to the data object content and the signature, metadata is also included
to describe different kind of Catalyst Signed Document properties.

## Motivation: why is this CIP necessary?

As we decentralize project catalyst, it will be required to unambiguously identify who produced some
data object, and the purpose of it.

## Specification

Catalyst Signed Document is [COSE] based structure, particularly `COSE Signed Data Object` [COSE] type.
It fully inherits an original [COSE] design and specifies the details of different [COSE] header's fields.

### Catalyst Signed Document metadata fields

To uniquely specify a Catalyst Signed Document type, version etc., as it was mentioned before,
a list of different metadata fields is specified.

Also as you can see from the specification,
it is allowed to add any number of additional metadata fields, which could be specified for each `type` of document.

[A full list of considered additional metadata fields](./meta.md).

All these fields will be encoded as the [COSE] `protected` header

<!-- markdownlint-disable max-one-sentence-per-line code-block-style -->
??? note "Catalyst Signed Document metadata fields: `signed_doc_meta.cddl`"

    ```CDDL
    {{ include_file('src/architecture/08_concepts/signed_doc/cddl/signed_doc_meta.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line code-block-style -->

#### `type`

Each Catalyst Signed Document will have a type identifier called `type`.

The `type` is a [UUID] v4.

[A full list of Catalyst supported document types](./types.md)

#### `id`

Every Catalyst Signed Document will have a unique ID.
All Catalyst Signed Document with the same `id` are considered versions of the same Catalyst Signed Document
(read about [`ver`](#ver)).
However, `id` uniqueness is only guaranteed on first use.

If the same `id` is used, by unauthorized publishers, the Catalyst Signed Document is invalid.

The `id` is a [UUID] v7.

The first time a Catalyst Signed Document is created, it will be assigned by the creator a new `id` which must
be well constructed.

* The time must be the time the Catalyst Signed Document was first created.
* The random value must be truly random.

Creating `id` this way ensures there are no collisions, and they can be independently created without central co-ordination.

*Note: All Catalyst Signed Documents are signed,
the first creation of an `id` assigns that `id` to the creator and any assigned collaborators.
A Catalyst Signed Document that is not signed by the creator, or an assigned collaborator, is invalid.
There is no reasonable way an `id` can collide accidentally.
Therefore, detection of invalid `id`s published by unauthorized publishers, could result in anti-spam
or system integrity mitigations being triggered.
This could result in all actions in the system being blocked by the offending publisher,
including all otherwise legitimate publications by the same author being marked as fraudulent.*

#### `ver`

Every Catalyst Signed Document in the system will be versioned.
There can, and probably will, exist multiple versions of the same document.

The `ver` is a [UUID] v7.

The initial `ver` assigned the first time a Catalyst Signed Document is published will be identical to the [`id`](#id).
Subsequent versions will retain the same [`id`](#id) and will create a new `ver`,
following best practice for creating a new [UUID] v7.

#### `alg`

This is an original [COSE] header field,
which indicates the cryptography algorithm used for the security processing.

<!-- markdownlint-disable max-one-sentence-per-line -->
!!! warning ""

    It must be equal to `EdDSA` value
<!-- markdownlint-enable max-one-sentence-per-line -->

Only `ed25119` considered at this moment as the only option to be supported for signed objects.

#### [`content type`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Type)

This is an original [COSE] header field,
which indicates the [`content type`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Type)
of the [content](#catalyst-signed-document-content) ([COSE] `payload`) data.

#### [`content encoding`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Encoding) (optional)

This field is used to indicate the [`content encoding`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Encoding)
algorithm of the [content](#catalyst-signed-document-content) data.

Supported encodings:

* `br` - [Brotli] compressed data.

### Catalyst Signed Document content

The Catalyst Signed Document content data is encoded (and could be additionally compressed,
read [`content encoding`](#content-encoding-optional)) as [COSE] `payload`.

### [COSE] signature protected header

As it mentioned earlier, Catalyst Signed Document utilizes `COSE Signed Data Object` format,
which allows to provide multi-signature functionality.
In that regard,
each Catalyst Signed Document [COSE] signature **must** include the following `protected` header field:

<!-- markdownlint-disable code-block-style -->
```CDDL
; All encoders/decoders of this specification must follow deterministic cbor encoding rules
; https://datatracker.ietf.org/doc/html/draft-ietf-cbor-cde-06

signature_protected_header = {
    4 => bytes ; "kid", UTF-8 encoded URI string
}
```
<!-- markdownlint-enable code-block-style -->

* `kid`: A unique identifier of the signer.
  A [UTF-8] encoded [URI] string.

## Copyright

This document is licensed under [CC-BY-4.0](https://creativecommons.org/licenses/by/4.0/legalcode).

[Brotli]: https://datatracker.ietf.org/doc/html/rfc7932
[UTF-8]: https://datatracker.ietf.org/doc/html/rfc3629
[URI]: https://datatracker.ietf.org/doc/html/rfc3986
[COSE]: https://datatracker.ietf.org/doc/html/rfc9052
[UUID]: https://www.rfc-editor.org/rfc/rfc9562.html
