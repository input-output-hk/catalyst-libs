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

While every Catalyst Signed Document is a valid [COSE Sign][RFC9052-CoseSign] format document,
not every [COSE Sign][RFC9052-CoseSign] format document is a valid Catalyst Signed Document.
The following restrictions apply:

### Unprotected Headers are not permitted

It is a requirement that any document that contains exactly the same data, must produce the same
catalyst signed document.
This means that unprotected headers, which do not form part of the data protected by
the signature are not permitted.
Any document which contains any unprotected headers is not a valid Catalyst Signed Document,
even though it may be a valid [COSE Sign][RFC9052-CoseSign] formatted document.

### Only defined metadata and [COSE][RFC9052] Headers are allowed

Each document type, defines a set of metadata and the [COSE][RFC9052] Headers which are allowed in that document type.
Even if the Catalyst Signed document metadata exists in this specification, IF it is not defined as
a valid metadata or [COSE][RFC9052] Header field for that particular document it may not be present.
Unexpected but otherwise valid Metadata or [COSE][RFC9052] Header fields invalidate the Catalyst Signed Document.

### No undefined metadata or unused [COSE][RFC9052] Headers may be present

[COSE][RFC9052] Header Fields which are defined by the [COSE][RFC9052] Specification, but are NOT defined as part of a
Catalyst Signed Document may not be present.
Any such [COSE][RFC9052] Header Fields present in the document render it an invalid Catalyst Signed Document.

Any metadata field that is not defined in this specification may not be present in any protected header.
Unrecognized metadata fields in a document render it an invalid Catalyst Signed Document.

### [CBOR Deterministic Encoding][CBOR-LFD-ENCODING] MUST be used

The Catalyst Signed Document **MUST** be encoded using [CBOR Deterministic Encoding][CBOR-LFD-ENCODING].
The "length-first core deterministic encoding requirements" variant of deterministic encoding *MUST* be used.

### Signed Document [CDDL][RFC8610] Definition

<!-- markdownlint-disable max-one-sentence-per-line -->
??? note "CDDL Specification"

    * [signed_document.cddl](cddl/signed_document.cddl)

    ``` cddl
    {{ include_file('./cddl/signed_document.cddl', indent=4) }}
    ```

<!-- markdownlint-enable max-one-sentence-per-line -->

### [COSE Header Parameters][RFC9052-HeaderParameters]

[COSE][RFC9052] documents define a set of standard [COSE header parameters][RFC9052-HeaderParameters].
All [COSE Header Parameters][RFC9052-HeaderParameters] are protected and
*MUST* appear in the protected headers section of the document.
The [COSE header parameters][RFC9052-HeaderParameters] defined and used by Catalyst Signed Documents are as follows:

#### `content type`

Media Type/s allowed in the Payload

* Required : yes
* [Cose][RFC9052] Label : 3
* Format : Media Type
  * Supported Values:
    * [application/cbor] :
      An [RFC8949] Binary [CBOR][RFC8949] Encoded Document.
    * [application/cddl][RFC8610] :
      A [CDDL][RFC8610] Document.

      Note:

      * This is an unofficial media type
      * [RFC9165] Additional Control Operators for [CDDL][RFC8610] are supported.
      * Must not have Modules, schema must be self-contained.
    * [application/json] :
      [JSON][RFC8259] Document
    * [application/schema+json][JSON Schema-2020-12] :
      A [JSON Schema Draft 2020-12][JSON Schema-2020-12] Document.

      Note:

      * This is currently an unofficial media type.
    * [text/css;][text/css] [charset=utf-8][RFC3629] :
      [CSS] Content used for styling [HTML][HTML5].
      [CSS] should use the least set of features possible to achieve
      the desired presentation to ensure the broadest compatibility.
    * [text/css;][text/css] [charset=utf-8;][RFC3629] [template=handlebars][Handlebars] :
      [CSS] Content used for styling [HTML][HTML5].
      [CSS] should use the least set of features possible to achieve
      the desired presentation to ensure the broadest compatibility.

      The text includes [Handlebars] type template fields that need
      processing and replacement prior to display.
    * [text/html;][HTML5] [charset=utf-8][RFC3629] :
      Formatted text using [HTML5] markup for rich text.
      Only [HTML5] syntax is supported.
    * [text/html;][HTML5] [charset=utf-8;][RFC3629] [template=handlebars][Handlebars] :
      Formatted text using [HTML5] markup for rich text.
      Only [HTML5] syntax is supported.

      The text includes [Handlebars] type template fields that need
      processing and replacement prior to display.
    * [text/markdown;][CommonMark] [charset=utf-8][RFC3629] :
      Formatted text using [Markdown][CommonMark] for rich text.
      [Markdown][CommonMark] formatting is as defined by [CommonMark].

      IF the document includes HTML, then [HTML5] syntax only is supported.

      The following [Markdown][CommonMark] Extensions are also supported:

      * None
    * [text/markdown;][CommonMark] [charset=utf-8;][RFC3629] [template=handlebars][Handlebars] :
      Formatted text using [Markdown][CommonMark] for rich text.
      [Markdown][CommonMark] formatting is as defined by [CommonMark].

      IF the document includes HTML, then [HTML5] syntax only is supported.

      The following [Markdown][CommonMark] Extensions are also supported:

      * None

      The text includes [Handlebars] type template fields that need
      processing and replacement prior to display.
    * [text/plain;][text/plain] [charset=utf-8][RFC3629] :
      Plain Text with no markup or special formatting.
      Multiline Plain Text *MUST* always interpret `
      `
      as a hard line break.
    * [text/plain;][text/plain] [charset=utf-8;][RFC3629] [template=handlebars][Handlebars] :
      Plain Text with no markup or special formatting.
      Multiline Plain Text *MUST* always interpret `
      `
      as a hard line break.

      The text includes [Handlebars] type template fields that need
      processing and replacement prior to display.

#### `content-encoding`

Supported HTTP Encodings of the Payload.
If no compression or encoding is used, then this field must not be present.

* Required : optional
* [Cose][RFC9052] Label : content-encoding ***Custom Header***
* Format : HTTP Content Encoding
  * Supported Values:
    * [br] :
      [BROTLI][RFC7932] Compression

### Metadata

Catalyst Signed Documents extend the Header Parameters with a series of [Metadata fields](./metadata.md).

### Signing Catalyst Signed Documents

Catalyst Signed Documents are based on the [COSE Sign][RFC9052-CoseSign] format.
This allows one or more signatures to be attached to the same document.
A catalyst signed document *MUST* have at least one valid signature attached.
Multiple signatures may also be attached to the same document, where that is required.

Each signature is contained in an array of signatures attached to the document.
The signatures contain protected headers, and the signature itself.
The headers currently defined for the signatures are:

#### `kid`

Catalyst ID [URI][RFC3986] identifying the Public Key.

The `kid` is a [UTF-8][RFC3629] encoded Catalyst ID [URI][RFC3986].
Any `kid` [URI][RFC3986] which conforms to the Catalyst ID specification may be used.
The Catalyst ID unambiguously defines both the signing keys and signing algorithm
used to sign the protected portion of the document.

* Required : yes
* [Cose][RFC9052] Label : 4
* Format : Catalyst ID

## Copyright

| Copyright | :copyright: 2024-2025 IOG Singapore, All Rights Reserved |
| --- | --- |
| License | This document is licensed under [CC-BY-4.0] |
| Created | 2024-12-27 |
| Modified | 2025-05-30 |
| Authors | Alex Pozhylenkov <alex.pozhylenkov@iohk.io> |
| | Steven Johnson <steven.johnson@iohk.io> |

### Changelog

#### 0.01 (2025-04-04)

* First Published Version

#### 0.02 (2025-04-09)

* Add version control changelogs to the specification.

#### 0.03 (2025-05-05)

* Use generalized parameters.

#### 0.04 (2025-05-30)

* Improve and make document serialization more repeatable, and stricter.
* TODO: Define Systems parameters
* TODO: Define DReps documents.
* TODO: Define Proposer Profiles.
* TODO: Define Role 0 Profile.

[CBOR-LFD-ENCODING]: https://www.rfc-editor.org/rfc/rfc8949.html#section-4.2.3
[JSON Schema-2020-12]: https://json-schema.org/draft/2020-12
[RFC9052-HeaderParameters]: https://www.rfc-editor.org/rfc/rfc8152#section-3.1
[Handlebars]: https://handlebarsjs.com/
[RFC8610]: https://www.rfc-editor.org/rfc/rfc8610
[application/cbor]: https://www.iana.org/assignments/media-types/application/cbor
[application/json]: https://www.iana.org/assignments/media-types/application/json
[RFC3629]: https://datatracker.ietf.org/doc/html/rfc3629
[CommonMark]: https://spec.commonmark.org/0.31.2/
[text/plain]: https://www.rfc-editor.org/rfc/rfc2046.html
[HTML5]: https://html.spec.whatwg.org/multipage/syntax.html#syntax
[RFC9052-CoseSign]: https://datatracker.ietf.org/doc/html/rfc9052#name-signing-with-one-or-more-si
[text/css]: https://www.rfc-editor.org/rfc/rfc2318.html
[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
[RFC8949]: https://www.rfc-editor.org/rfc/rfc8949.html
[RFC9165]: https://www.rfc-editor.org/rfc/rfc9165
[RFC7932]: https://www.rfc-editor.org/rfc/rfc7932
[RFC9052]: https://datatracker.ietf.org/doc/html/rfc9052
[RFC8259]: https://www.rfc-editor.org/rfc/rfc8259.html
[RFC3986]: https://datatracker.ietf.org/doc/html/rfc3986
[CSS]: https://www.w3.org/Style/CSS/
[br]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Encoding#br
