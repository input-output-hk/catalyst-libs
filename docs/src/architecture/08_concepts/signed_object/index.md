---
Title: Catalyst Signed Object
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
  * [Signed Object fields](#signed-object-fields)
    * [`type`](#type)
    * [`id`](#id)
    * [`ver`](#ver)
    * [`alg`](#alg)
    * [`content type`](#content-type)
    * [`content encoding` (optional)](#content-encoding-optional)
  * [Signed Object content](#signed-object-content)
  * [COSE signature protected header](#cose-signature-protected-header)
* [Copyright](#copyright)

## Abstract

Project Catalyst both produces and consumes a lot of different data objects,
in different places of the system.
To ensure the data object is authoritative, it must be signed.
Id addition to the data object content and the signature, metadata is also included
to describe different kind of signed object properties.

## Motivation: why is this CIP necessary?

As we decentralize project catalyst, it will be required to unambiguously identify who produced some
data object, and the purpose of it.

## Specification

Catalyst signed object is [COSE] based structure, particularly `COSE Signed Data Object` [COSE] type.
It fully inherits an original [COSE] design and specifies the details of different [COSE] header's fields.

### Signed Object fields

To uniquely specify a signed object type, version etc., as it was mentioned before,
a list of different fields is specified.
Also as you can see from the specification,
it is allowed to add any number of additional metadata fields, which could be specified for each `type` of signed object.

All these fields will be encoded as the [COSE] `protected` header

<!-- markdownlint-disable max-one-sentence-per-line code-block-style -->
??? note "Catalyst signed object fields: `signed_object_meta.cddl`"

    ```CDDL
    {{ include_file('src/architecture/08_concepts/signed_object/cddl/signed_object_meta.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line code-block-style -->

#### `type`

Each signed object will have a type identifier called `type`.

The `type` is a [UUID] V4.

#### `id`

Every signed object will have a unique ID.
All signed object with the same `id` are considered versions of the same signed object
(read about [`ver`](#ver)).
However, `id` uniqueness is only guaranteed on first use.

If the same `id` is used, by unauthorized publishers, the signed object is invalid.

The `id` is a [UUID] V7.

The first time a signed object is created, it will be assigned by the creator a new `id` which must
be well constructed.

* The time must be the time the signed object was first created.
* The random value must be truly random.

Creating `id` this way ensures there are no collisions, and they can be independently created without central co-ordination.

*Note: All signed objects are signed, the first creation of an `id` assigns that `id` to the creator and any assigned collaborators.
A Signed Object that is not signed by the creator, or an assigned collaborator, is invalid.
There is no reasonable way an `id` can collide accidentally.
Therefore, detection of invalid `id`s published by unauthorized publishers, could result in anti-spam
or system integrity mitigations being triggered.
This could result in all actions in the system being blocked by the offending publisher,
including all otherwise legitimate publications by the same author being marked as fraudulent.*

#### `ver`

Every signed object in the system will be versioned.
There can, and probably will, exist multiple versions of the same document.

The `ver` is a [UUID] V7.

The initial `ver` assigned the first time a signed object is published will be identical to the [`id`](#id).
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

#### `content type`

This is an original [COSE] header field,
which indicates the `content type` of the [content](#signed-object-content) ([COSE] `payload`) data.

#### `content encoding` (optional)

This field is used to indicate the content encodings algorithm of the [content](#signed-object-content) data.

### Signed Object content

The signed object content data is encoded (and could be additionally compressed,
read [`content encoding`](#content-encoding-optional)) as [COSE] `payload`.

### [COSE] signature protected header

As it mentioned earlier, Catalyst signed document utilizes `COSE Signed Data Object` format,
which allows to provide multi-signature functionality.
In that regard,
each Catalyst signed object [COSE] signature **must** include the following `protected` header field:

<!-- markdownlint-disable code-block-style -->
```CDDL
; All encoders/decoders of this specification must follow deterministic cbor encoding rules
; https://datatracker.ietf.org/doc/html/draft-ietf-cbor-cde-06

signature_protected_header = {
    4 => bytes ; "kid"
}
```
<!-- markdownlint-enable code-block-style -->

* `kid`: A unique identifier of the signer.

## Copyright

This document is licensed under [CC-BY-4.0](https://creativecommons.org/licenses/by/4.0/legalcode).

[COSE]: https://datatracker.ietf.org/doc/html/rfc9052
[UUID]: https://www.rfc-editor.org/rfc/rfc9562.html
