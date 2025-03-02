<!-- cspell: words collabs -->

# Catalyst signed document

Catalyst signed document crate implementation based on this
[spec](https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/spec/)

## Example

### Generate a `ed25519` private and public keys

```shell
openssl genpkey -algorithm=ED25519 -out=private.pem -outpubkey=public.pem
```

### Prepare non-signed document

`meta.json` file should follow the [`meta.schema.json`](./meta.schema.json).

```shell
cargo run -p catalyst-signed-doc build signed_doc/doc.json signed_doc/doc.cose signed_doc/meta.json
```

### Sign document

`KID` is a valid Catalyst ID URI.

```shell
cargo run -p catalyst-signed-doc sign signed_doc/doc.cose signed_doc/meta.json <KID>
```

### Inspect document

```shell
cargo run -p catalyst-signed-doc inspect signed_doc/doc.cose
```
