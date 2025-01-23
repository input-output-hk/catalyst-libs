<!-- cspell: words collabs -->

# Catalyst signed document

Catalyst signed document crate implementation based on this
[spec](https://input-output-hk.github.io/catalyst-libs/architecture/08_concepts/signed_doc/spec/)

## Example

Generate a `ed25519` private and public keys

```shell
openssl genpkey -algorithm=ED25519 -out=private.pem -outpubkey=public.pem
```

Prepare non-signed document,
`meta.json` file should follow the [`meta.schema.json`](./meta.schema.json).

```shell
cargo run -p catalyst-signed-doc --example mk_signed_doc build signed_doc/doc.json signed_doc/doc.cose signed_doc/meta.json
```

Inspect document

```shell
cargo run -p catalyst-signed-doc --example mk_signed_doc inspect signed_doc/doc.cose
```
