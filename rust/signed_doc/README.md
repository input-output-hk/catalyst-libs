# Catalyst signed document

Catalyst signed document is [COSE] based document structure,
particularly `COSE Signed Data Object` [COSE] type.

## Structure

This document structure is fully inherits an original [COSE] design and specifies the details
of different [COSE] header's fields.

### Protected header

The [COSE] standard defines two types of headers: `protected` and `unprotected`.
C-COSED specifies the following `protected` header fields, which **must** be present:

* `alg`: `EdDSA`
  (this parameter is used to indicate the algorithm used for the security processing,
  in this particular case `ed25119` signature algorithm is used).
* `content type`: `application/json`
  (this parameter is used to indicate the content type of the payload data,
  in this particular case `JSON` format is used).
* `content encoding`: `br`
  (this parameter is used to indicate the content encodings algorith of the payload data,
  in this particular case [brotli] compression data format is used).
* `id`
* `ver`
* `ref`
* ...

### C-COSED payload

The [COSE] signature payload, as mentioned earlier,
the content type of the [COSE] signature payload is JSON, [brotli] compressed.
Which stores an actual document data which should follow to some schema.

### Signature protected header

As it mentioned earlier, C-COSED utilizes `COSE Signed Data Object` format,
which allows to provide mutlisignature functionality.
In that regard, each C-COSED [COSE] signature **must** include the following protected header field:

`protected`:

* `kid`: a Blake2B hash of the signer's [x.509] certificate (ASN.1 DER encoded bytes) associated with its keys
  (this parameter identifies one piece of data
  that can be used as input to find the needed cryptographic key).

## CLI

Prepare non-signed document

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
