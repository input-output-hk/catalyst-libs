# CBOR Kit (`cbork`)

This is a CLI tool for operating with CDDL and CBOR.

It will grow over time to provide a number of features, and will be supported by individual `cbork-*` crates.

## Install

To install this tool run

```shell
cargo install --git https://github.com/input-output-hk/catalyst-libs.git cbork
```

## Features

### CDDL linter

[CDDL](#cddl-specifications) (Concise Data Definition Language)
linting CLI tool.
Enables users to check their CDDL code for errors, inconsistencies, and compliance with the CDDL specification.

#### Currently supports

* [CDDL][1]
* [CDDL Errata][2]
* [CDDL Extensions][3]

#### Planned support for

* [CDDL Modules][4]
* [CDDL Module Standard Library][5]

## Planned Future Tools within the CLI

* [ ] A tool to generate a Rust module to decode/encode/validate Data efficiently from a [CDDL](#cddl-specifications) definition.
* [ ] A tool to simply validate [CBOR][6] binary data against a [CDDL](#cddl-specifications) definition.

## Notes

There are semantic rules about well-formed [CDDL](#cddl-specifications) files that are not enforced by the grammar.
The full parser does not support these rules currently, but is planned to be extended to validate
those rules in future

The primary rule that is not currently enforced is that the very first definition in the file is the base type.
We also cannot detect orphaned or missing definitions.

Both of these are planned for a future release of the tool.

There may be other checks needed to be performed on the parsed AST for validity.

## CDDL Specifications

* [RFC8610][1]
* [RFC8610 Errata][2]
* [RFC9165 CDDL Extensions][3]
* [CDDL Modules][4]
* [CDDL Modules Reference][5]

[1]: https://www.rfc-editor.org/rfc/rfc8610 "RFC-8610"
[2]: https://www.ietf.org/archive/id/draft-ietf-cbor-update-8610-grammar-01.html "RFC-8610 Errata"
[3]: https://www.rfc-editor.org/rfc/rfc9165 "RFC-9165 CDDL Extensions"
[4]: https://cbor-wg.github.io/cddl-modules/draft-ietf-cbor-cddl-modules.html "CDDL Modules Specification"
[5]: https://github.com/cabo/cddlc "CDDL Modules Standard Library"
[6]: https://datatracker.ietf.org/doc/html/rfc8949 "RFC-8949 CBOR Data Interchange Format"
