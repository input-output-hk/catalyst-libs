# Metadata Fields 

## `ref` Document Reference

This is a reference to another document.
The purpose of the `ref` will vary depending on the document [`type`](./spec.md#type).

The `ref` can be either a single [UUID] or a [CBOR] Array of Two [UUID].

If the `ref` is a single [UUID] v7, it is a reference to the document of that [`id`](./spec.md#id).
If the `ref` is a [CBOR] array, it has the form `[<id>,<ver>]` where:

* `<id>` - the [UUID] v7 of the referenced documents [`id`](./spec.md#id).
* `<ver>` - the [UUID] v7 of the referenced documents [`ver`](./spec.md#ver).

### Validation

For any document type, `ref` can refer to only 1 other document type which must be different than
the type of document `ref` appears in.
For example `ref` for a Proposal Comment Document, is always a Proposal type document.

## `ref_hash` Secured Document Reference

This is a cryptographically secured reference to another document.

It consists of two fields:

* [`ref`](#ref-document-reference) - simple reference to the document.
* `hash` - hash of the referenced document [CBOR] bytes.

## `template` Template Reference

If the document was formed from a template, this is a reference to that template document.
The format is the same as the [CBOR] Array form of [`ref`](#ref-document-reference).

It is invalid not to reference the template that formed a document.
If this is missing from such documents, the document will itself be considered invalid.

Template references must explicitly reference both the Template Document ID, and Version.

## `reply` Reply Reference

This is a reply to another document.
The format is the same as the [CBOR] Array form of [`ref`](#ref-document-reference).

`reply` is always referencing a document of the same type, and that document must `ref` reference the same document `id`.
However, depending on the document type, it may reference a different `ver` of that `id`.

## `section` Section Reference

This is a reference to a section of a document.
It is a CBOR String, and contains a [JSON Path] identifying the section in question.

Because this metadata field uses [JSON Path], it can only be used to reference a document whose payload is [JSON].

## `collabs` Authorized Collaborators

This is a list of entities other than the original author that may also publish versions of this document.
This may be updated by the original author, or any collaborator that is given "author" privileges.

The latest `collabs` list in the latest version, published by an authorized author is the definitive
list of allowed collaborators after that point.

The `collabs` list is a [CBOR] Array.
The contents of the array are TBD.

However, they will contain enough information such that each collaborator can be uniquely identified and validated.

*Note: An Author can unilaterally set the `collabs` list to any list of collaborators.
It does NOT mean that the listed collaborators have agreed to collaborate, only that the Author
gives them permission to.*

This list can impact actions that can be performed by the `Proposal Action Document`.

## `brand_id`

This is a reply to another document.
The format is the same as the [CBOR] Array form of [`ref`](#ref-document-reference).

`brand_id` represents a "brand" who is running the voting, e.g. Catalyst, Midnight.

## `campaign_id`

This is a reply to another document.
The format is the same as the [CBOR] Array form of [`ref`](#ref-document-reference).

`campaign_id` defines a "campaign" of voting, e.g. "treasury campaign".

## `election_id`

Unique identifier [UUID] v4, which defines an election,
e.g. "Catalyst Fund 1", "Catalyst Fund 2".

## `category_id`

This is a reply to another document.
The format is the same as the [CBOR] Array form of [`ref`](#ref-document-reference).

`campaign_id` defines a voting category as a collection of proposals,
e.g. "Development & Infrastructure", "Products & Integrations".

[UUID]: https://www.rfc-editor.org/rfc/rfc9562.html
[CBOR]: https://datatracker.ietf.org/doc/rfc8949/
[JSON]: https://datatracker.ietf.org/doc/html/rfc7159
[JSON Path]: https://datatracker.ietf.org/doc/html/rfc9535
