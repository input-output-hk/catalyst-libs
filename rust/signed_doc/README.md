<!-- cspell: words collabs -->

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
  (this parameter is used to indicate the content encodings algorithm of the payload data,
  in this particular case [brotli] compression data format is used).
* `type`: CBOR encoded UUID.
* `id`: CBOR encoded ULID.
* `ver`: CBOR encoded ULID `#6.32780(bytes)`.
* `ref`: CBOR encoded ULID or two elements array of ULIDs (optional).
* `template`: CBOR encoded ULID or two elements array of ULIDs (optional).
* `reply`: CBOR encoded ULID or two elements array of ULIDs (optional).
* `section`: CBOR encoded string (optional).
* `collabs`: CBOR encoded array of any CBOR types (optional).

Precise CDDL definition

```cddl
protected_header = {
   1 => -8, ; "alg": EdDSA
   3 => 30, ; "content type": Json
   "content encoding" => "br", ; payload content encoding, brotli compression
   "type" => UUID
   "id" => ULID
   "ver" => ULID
   ? "ref" => reference_type
   ? "template" => reference_type
   ? "reply" => reference_type
   ? "section" => text,
   ? "collabs" => [+any],
}

UUID = #6.37(bytes)
ULID = #6.32780(bytes)
reference_type = ULID / [ULID, ULID] ; either ULID or [ULID, ULID]
```

### COSE payload

The [COSE] signature payload, as mentioned earlier,
the content type of the [COSE] signature payload is JSON, [brotli] compressed.
Which stores an actual document data which should follow to some schema.

### Signature protected header

As it mentioned earlier, Catalyst signed document utilizes `COSE Signed Data Object` format,
which allows to provide mutli-signature functionality.
In that regard,
each Catalyst signed document [COSE] signature **must** include the following protected header field:

`protected`:

* `kid`: CBOR encoded `bytes` type.

Precise CDDL definition

```cddl
signature_protected_header = {
    4 => bytes ; "kid"
}
```

## Example

Generate a `ed25519` private and public keys

```shell
openssl genpkey -algorithm=ED25519 -out=private.pem -outpubkey=public.pem
```

Prepare non-signed document,
`meta.json` file should follow the [`meta.schema.json`](./meta.schema.json).

```shell
cargo run -p signed_doc --example mk_signed_doc build
signed_doc/doc.json  signed_doc/schema.json signed_doc/doc.cose signed_doc/meta.json
```

Sign document

```shell
cargo run -p signed_doc --example mk_signed_doc sign private.pem signed_doc/doc.cose kid_1
```

Verify document

```shell
cargo run -p signed_doc --example mk_signed_doc verify
public.pem signed_doc/doc.cose signed_doc/schema.json
```

Catalyst signed document CBOR bytes example

```cbor
845861A6012703183270636F6E74656E7420656E636F64696E676262726474797065D825500CE8AB3892584FBCA62E7F
AA6E58318F626964D9800C500193929C1D227F1977FED19443841F0B63766572D9800C500193929C1D227F1977FED194
43841F0BA0584E1B6D00209C05762C9B4E1EAC3DCA9286B50888CBDE8E99A2EB532C3A0D83D6F6462707ECDFF7F9B74B
8904098479CA4221337F7DB97FDA25AFCC10ECB75722C91A485AAC1158BA6F90619221066C828347A104446B696431A0
584090DF51433D97728ACF3851C5D3CA2908F76589EA925AF434C5619234E4B1BA7B12A124EA79503562B33214EBC730
C9837E1CA909BB8163D7904B09C3FD6A5B0B8347A104446B696432A05840AB318FEF3FF46E69E760540B0B44E9E8A51A
84F23EC8A870ECDEBF9AD98EBB8212EBE5EA5FDBA87C98DF8DF259BE7873FE8B9EB54CC6558337B5C95D90CC3504
```

[COSE]: https://datatracker.ietf.org/doc/html/rfc9052
[brotli]: https://datatracker.ietf.org/doc/html/rfc7932
