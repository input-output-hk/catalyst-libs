# Catalyst Signed Document Specification

## Abstract

Project Catalyst requires a verifiable data format for the publication and validation of
large volumes of off chain information.

The Catalyst Signed Documents Specification is based on [COSE][RFC9052]
and provides the basis of this document specification.

## Motivation

As Project Catalyst decentralizes via both on-chain and off-chain mechanisms, a reliable,
standardized process for authenticating documents and their relationships is required.

## Specification

Project Catalyst generates a large volume of off chain information.
This information requires similar guarantees as on-chain data.
It needs to be verifiably published and also immutable.
However, we also require the ability to publish new versions of documents,
and for documents to be able to securely reference one another.

Catalyst Signed Documents are based on [COSE][RFC9052].
Specifically, the [COSE Sign][RFC9052-CoseSign] format is used.
This allows one or more signatures to be attached to the same document.

### Signed Document [CDDL][RFC8610] Definition

<!-- markdownlint-disable max-one-sentence-per-line -->
??? note "CDDL"

    * [cddl/signed_document.cddl](cddl/signed_document.cddl)

    ```cddl
    {{ include_file('./cddl/signed_document.cddl', indent=4) }}
    ```

<!-- markdownlint-enable max-one-sentence-per-line -->

### [COSE Header Parameters][RFC9052-HeaderParameters]

[COSE][RFC9052] documents define a set of standard [COSE header parameters][RFC9052-HeaderParameters].
All [COSE Header Parameters][RFC9052-HeaderParameters] are protected and
*MUST* appear in the protected headers section of the document.
The [COSE header parameters][RFC9052-HeaderParameters] defined and used by Catalyst Signed Documents are as follows:

#### content type

IANA Media Type/s allowed in the Payload

* Required : yes
* [Cose][RFC9052] Label : 3
* Format : IANA Media Type
  * Supported Values:
    * [application/json] : [JSON][RFC8259] Document
    * [application/schema+json] : [JSON Schema] Draft 7 Document; Note:
      * This is currently an unofficial media type.
      * Draft 7 is used because of its wide support by tooling.
    * [application/cbor] : [RFC8949] Binary [CBOR][RFC8949] Encoded Document
    * application/cddl : [CDDL][RFC8610] Document; Note:
      * This is an unofficial media type
      * [RFC9165] Additional Control Operators for [CDDL][RFC8610] are supported.
      * Must not have Modules, schema must be self-contained.

#### content-encoding

Supported HTTP Encodings of the Payload.
If no compression or encoding is used, then this field must not be present.

* Required : optional
* [Cose][RFC9052] Label : content-encoding ***Custom Header***
* Format : HTTP Content Encoding
  * Supported Values:
    * [br] : [BROTLI][RFC7932] Compression

### Metadata

Catalyst Signed Documents extend the Header Parameters with a series of Metadata fields.
These fields are defined [here](./metadata.md).

### Signing Catalyst Signed Documents

Catalyst Signed Documents are based on the [COSE Sign][RFC9052-CoseSign] format.
This allows one or more signatures to be attached to the same document.
A catalyst signed document *MUST* have at least one valid signature attached.
Multiple signatures may also be attached to the same document, where that is required.

Each signature is contained in an array of signatures attached to the document.
The signatures contain protected headers, and the signature itself.
The headers currently defined for the signatures are:

#### `kid`

The kid is a [UTF-8][RFC3629] encoded Catalyst ID.
Any `kid` format which conforms to the Catalyst ID specification may be used.
The Catalyst ID unambiguously defines both the signing keys and signing algorithm
used to sign the protected portion of the document.

* Required: yes
* [Cose][RFC9052] Label: 4
* Format: [UTF-8][RFC3629] encoded Catalyst ID

## Copyright

| Copyright | :copyright: 2024-2025 IOG Singapore, All Rights Reserved |
| --- | --- |
| License | This document is licensed under [CC-BY-4.0] |
| Created | 2024-12-27 |
| Modified | 2025-04-09 |
| Authors | Alex Pozhylenkov <alex.pozhylenkov@iohk.io> |
| | Steven Johnson <steven.johnson@iohk.io> |

### Changelog

#### 0.01 (2025-04-04)

* First Published Version

#### 0.02 (2025-04-09)

* Add version control changelogs to the specification.

[application/schema+json]: https://datatracker.ietf.org/doc/draft-bhutton-json-schema/
[RFC9052-HeaderParameters]: https://www.rfc-editor.org/rfc/rfc8152#section-3.1
[application/cbor]: https://www.iana.org/assignments/media-types/application/cbor
[application/json]: https://www.iana.org/assignments/media-types/application/json
[JSON Schema]: https://json-schema.org/draft-07
[RFC9052-CoseSign]: https://datatracker.ietf.org/doc/html/rfc9052#name-signing-with-one-or-more-si
[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
[RFC8949]: https://www.rfc-editor.org/rfc/rfc8949.html
[RFC9165]: https://www.rfc-editor.org/rfc/rfc9165
[RFC7932]: https://www.rfc-editor.org/rfc/rfc7932
[RFC3629]: https://datatracker.ietf.org/doc/html/rfc3629
[RFC8610]: https://www.rfc-editor.org/rfc/rfc8610
[RFC9052]: https://datatracker.ietf.org/doc/html/rfc9052
[RFC8259]: https://www.rfc-editor.org/rfc/rfc8259.html
[br]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Encoding#br
