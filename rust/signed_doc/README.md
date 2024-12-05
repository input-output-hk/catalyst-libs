# Catalyst signed document

Catalyst signed document is [COSE] based document structure,
particularly `COSE Signed Data Object` [COSE] type.

## Structure

This document structure is fully inherits an original [COSE] design and specifies the details
of different [COSE] header's fields.

### Protected header

The [COSE] standard defines two types of headers: `protected` and `unprotected`.
Catalyst signed document specifies the following `protected` header fields,
which **must** be present (most of the fields originally defined by this
[spec](https://input-output-hk.github.io/catalyst-voices/architecture/08_concepts/signed_document_metadata/metadata/)):

* `alg`: `EdDSA`
  (this parameter is used to indicate the algorithm used for the security processing,
  in this particular case `ed25119` signature algorithm is used).
* `content type`: `application/json`
  (this parameter is used to indicate the content type of the payload data,
  in this particular case `JSON` format is used).
* `content encoding` (CBOR type `text`): `br` CBOR type `text`
  (this parameter is used to indicate the content encodings algorith of the payload data,
  in this particular case [brotli] compression data format is used).
* `type` (CBOR type `text`): CBOR encoded UUID `#6.37(bytes)`.
* `id` (CBOR type `text`): CBOR encoded ULID `#6.32780(bytes)`.
* `ver` (CBOR type `text`): CBOR encoded ULID `#6.32780(bytes)`.
* `ref` (CBOR type `text`): CBOR encoded ULID `#6.32780(bytes)`
  or array of ULIDs `[#6.32780(bytes), #6.32780(bytes)]`.
* `template` (CBOR type `text`): CBOR encoded ULID `#6.32780(bytes)`
  or array of ULIDs `[#6.32780(bytes), #6.32780(bytes)]`.
* `reply` (CBOR type `text`): CBOR encoded ULID `#6.32780(bytes)`
  or array of ULIDs `[#6.32780(bytes), #6.32780(bytes)]`.
* `section` (CBOR type `text`): CBOR encoded string, type `text`.
* `collabs` (CBOR type `text`): CBOR encoded array of any CBOR types `[+ any]`.

### COSE payload

The [COSE] signature payload, as mentioned earlier,
the content type of the [COSE] signature payload is JSON, [brotli] compressed.
Which stores an actual document data which should follow to some schema.

### Signature protected header

As it mentioned earlier, Catalyst signed document utilizes `COSE Signed Data Object` format,
which allows to provide mutlisignature functionality.
In that regard,
each Catalyst signed document [COSE] signature **must** include the following protected header field:

`protected`:

* `kid`: any string, CBOR encoded `text` type.

## Example

Generate a `ed25519` private and public keys

```shell
openssl genpkey -algorithm=ED25519 -out=private.pem -outpubkey=public.pem
```

Prepare non-signed document,
`meta.json` file should follow the [`meta.schema.json`](./meta.schema.json).

```shell
cargo run -p signed_doc --example mk_signed_doc build signed_doc/doc.json  signed_doc/schema.json signed_doc/doc.cose signed_doc/meta.json
```

Sign document

```shell
cargo run -p signed_doc --example mk_signed_doc sign private.pem signed_doc/doc.cose kid_1
```

Verify document

```shell
cargo run -p signed_doc --example mk_signed_doc verify public.pem signed_doc/doc.cose signed_doc/schema.json
```

[COSE]: https://datatracker.ietf.org/doc/html/rfc9052
[brotli]: https://datatracker.ietf.org/doc/html/rfc7932
