# Rep Nomination

## Description

A Representative Nomination Document is created to opt
in as a Representative Voter for a specific Contest on a Brand/Campaign or Category.

This Document is a kind of `Profile` that is primarily used to
help justify the Representatives Nomination to prospective delegators.

The user must have registered as a Representative.
The presence of this document signifies the user's intent to participate in that
contest as a Representative.

The document's structure is defined by the associated
Rep Nomination Form Template.
This allows an Admin to specify contest-specific requirements.

The Representative can retract their nomination by using the `revoke` metadata to
revoke this Nomination document.

It is an extension of all other profiles attached to the same Catalyst ID.

Profiles themselves are intentionally general, however they may be
linked to a Brand/Campaign/Category via the template used by the profile.

The payload of a profile is controlled by its template.

<!-- markdownlint-disable max-one-sentence-per-line -->

```graphviz dot rep_nomination.dot.svg

{{ include_file('./../diagrams/rep_nomination.dot', indent=4) }}
```

<!-- markdownlint-enable max-one-sentence-per-line -->

### Validation

* The signer MUST be a registered 'Representative'.
* The 'ref' metadata field MUST point to a valid 'Representative Profile' document.
* The 'parameters' metadata field MUST point to a valid 'Contest Parameters' document.
* The 'template' metadata field MUST point to a valid 'Representative Nomination Form Template' document.
* The payload MUST be valid against the [JSON schema][JSON Schema-2020-12] defined in the referenced template.
* Only **ONE** major version (same `id`) could be submitted per contest.
    If representative already submitted nomination for the specific contest,
    only sub-versions could be submitted by that representative
    (same [`id`](../metadata.md#id) different `ver`).
* Other rules may apply as defined by the Contest or other parameters which can
    control who may validly nominate as a representative in a Contest.

No Nomination is valid unless the latest Contest Delegation of the Delegate
refers to their own Nomination.
This requires that Nominating is a two step process:

1. Post the Nomination Document.
2. Post a Contest Delegation delegating to the new Nomination Document.

Updating the Nomination Document will invalidate all Nominations to the
Representative.

This is because Delegation points to a *SPECIFIC* Nomination, and it
*MUST* be the latest for the Representative on the Contest.
As the Nomination contains information that the User relies on
when choosing to delegate, changing that information could have a
real and detrimental result in the Delegation choice.
Therefore, for a Delegation to be valid, it *MUST* point to the
latest Nomination for a Representative.

Publishing a newer version of the Nomination Document to a specific contest will
invalidate all pre-existing delegations, and all voters will need
to re-delegate to affirm the delegates latest nomination.

### Business Logic

#### Front End

* Allows a Representative to create or update their profile for a category.
* The Representative sets their status to 'active' to be discoverable for delegation.
* The Representative `revokes` the Nomination to signal they are no longer
  participating in the category.
* Nominations are not valid if the latest Delegation by the Representative does NOT
  reference their latest Nomination.

#### Back End

* The backend MUST verify the signer is a 'Representative' and that all referenced documents exist.
* Only **ONE** major version (same `id`) could be submitted per 'Representative'.
* The system will only consider Representatives as having valid Nominations if:
    * Their latest Nomination in a Contest is not Revoked.
    * Their latest Delegation in a Contest references their latest Nomination.

## [COSE Header Parameters][RFC9052-HeaderParameters]

* [content type](../spec.md#content-type) = `application/json`
* [content-encoding](../spec.md#content-encoding) = `[br]`

## Metadata

### [`type`](../metadata.md#type)

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Type](../metadata.md#document-type) |
| Type | bf9abd97-5d1f-4429-8e80-740fea371a9c |
<!-- markdownlint-enable MD033 -->
The document TYPE.

#### [`type`](../metadata.md#type) Validation

**MUST** be a known document type.

### [`id`](../metadata.md#id)

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Id](../metadata.md#document-id) |
<!-- markdownlint-enable MD033 -->
Document ID, created the first time the document is created.
This must be a properly created [UUIDv7][RFC9562-V7] which contains the
timestamp of when the document was created.

#### [`id`](../metadata.md#id) Validation

The document ID validation is performed based on timestamp thresholds:

* If `future_threshold` is configured,
the document [`id`](../metadata.md#id) cannot be too far in the future from the
current time.
* If `past_threshold` is configured, the document [`id`](../metadata.md#id) cannot be too far in the past from the
current time.

### [`ver`](../metadata.md#ver)

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Ver](../metadata.md#document-ver) |
<!-- markdownlint-enable MD033 -->
The unique version of the document.
The first version of the document must set [`ver`](../metadata.md#ver) == [`id`](../metadata.md#id)

[`ver`](../metadata.md#ver) represents new versions of the same document as it changes over time.

#### [`ver`](../metadata.md#ver) Validation

1. The document version must always be >= the document ID.
2. IF [`ver`](../metadata.md#ver) does not == [`id`](../metadata.md#id)
  then a document with [`id`](../metadata.md#id) and [`ver`](../metadata.md#ver) being equal *MUST* exist.
3. When a document with the same [`id`](../metadata.md#id) already exists,
  the new document's [`ver`](../metadata.md#ver) must be greater than
  the latest known submitted version for that [`id`](../metadata.md#id).
4. When a document with the same [`id`](../metadata.md#id) already exists,
  the new document's [`type`](../metadata.md#type) must be the same as
  the latest known submitted document's [`type`](../metadata.md#type) for that [`id`](../metadata.md#id).

### [`ref`](../metadata.md#ref)

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Reference](../metadata.md#document-reference) |
| Valid References | [Rep Profile](rep_profile.md) |
<!-- markdownlint-enable MD033 -->
Reference to a Linked Document or Documents.
This is the primary hierarchical reference to a related document.

If a reference is defined as required, there must be at least 1 reference specified.
Some documents allow multiple references, and they are documented as required.

The document reference serves two purposes:

1. It ensures that the document referenced by an ID/Version is not substituted.
    In other words, that the document intended to be referenced, is actually referenced.
2. It Allows the document to be unambiguously located in decentralized storage systems.

There can be any number of Document Locations in any reference.
The currently defined locations are:

* `cid` : A [CBOR Encoded IPLD Content Identifier][CBOR-TAG-42] ( AKA an [IPFS CID][IPFS-CID] ).
* Others may be added when further storage mechanisms are defined.

The document location does not guarantee that the document is actually stored.
It only defines that if it were stored, this is the identifier
that is required to retrieve it.
Therefore it is required that Document References
are unique and reproducible, given a documents contents.

#### [`ref`](../metadata.md#ref) Validation

The following must be true for a valid reference:

* The Referenced Document **MUST** Exist
* Every value in the `document_locator` must consistently reference the exact same document.
* The `document_id` and `document_ver` **MUST** match the values in the referenced document.
* In the event there are **MULTIPLE** [`ref`](../metadata.md#ref) listed, they **MUST** be sorted.

Sorting for each element of [`ref`](../metadata.md#ref) follows the same sort order as specified for Map Keys,
as defined by [CBOR Deterministic Encoding][CBOR-LFD-ENCODING] (4.3.2 Length-First Map Key Ordering).

### [`template`](../metadata.md#template)

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Reference](../metadata.md#document-reference) |
| Valid References | [Rep Nomination Form Template](rep_nomination_form_template.md) |
<!-- markdownlint-enable MD033 -->
Reference to the template used to create and/or validate this document.

#### [`template`](../metadata.md#template) Validation

In addition to the validation performed for [Document Reference](../metadata.md#document-reference) type fields,
The document payload is not valid if it does not validate completely against the referenced template.

### [`revocations`](../metadata.md#revocations)

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | optional |
| Format | [Version Revocations](../metadata.md#version-revocations) |
<!-- markdownlint-enable MD033 -->
A document may include a list of any prior versions which are considered to be revoked.
Only the revocation list in the latest version of the document applies.
Revoked documents are flagged as no longer valid, and should not be displayed.
As a special case, if the revocations are set to `true` then all versions of the document
are revoked, including the latest document.

In this case, when the latest document is revoked, the payload may be `nil`.
Any older document that has [`revocations`](../metadata.md#revocations) set to `true` is always to be filtered
and its payload is to be assumed to be invalid.

This allows for an entire document and any/all published versions to be revoked.
A new version of the document that is published after this, may reinstate prior
document versions, by not listing them as revoked.
However, any document where revocations was set `true` can never be reinstated.

#### [`revocations`](../metadata.md#revocations) Validation

If the field is `true` the payload may be absent or invalid.
Such documents may never be submitted.

### [`parameters`](../metadata.md#parameters)

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Reference](../metadata.md#document-reference) |
| Valid References | [Contest Parameters](contest_parameters.md) |
| Linked Reference Metadata | [`template`](#template) |
<!-- markdownlint-enable MD033 -->
A reference to the Parameters Document this document lies under.

#### [`parameters`](../metadata.md#parameters) Validation

In addition to the validation performed for [Document Reference](../metadata.md#document-reference) type fields:

* Any linked referenced document that includes a [`parameters`](../metadata.md#parameters) metadata must match the
[`parameters`](../metadata.md#parameters) of the referencing document,
or a parent of those [`parameters`](../metadata.md#parameters).

For example, a linked reference to [Contest Parameters](contest_parameters.md) is transitively a reference to
the Parameters document it references, and each parameters document they reference
until the `Brand` parameters document is reached.

The use case here is for Templates.
The profile template, or proposal templates could be defined at any of these
levels, and as long as they all refer to the same chain of parameters in the
hierarchy they are all valid.

* The Document referenced by [`template`](../metadata.md#template)
    * MUST contain [`parameters`](../metadata.md#parameters) metadata; AND
    * MUST match the referencing documents [`parameters`](../metadata.md#parameters) value.

## Payload

The Representative's profile data for a specific contest.
Its structure is defined by the referenced template document.

In the case of Revoking a nomination the payload is `nil`.

## Signers

The following User roles may sign documents of this type:

* Representative

Updates are allowed by the original author OR from the 'collaborators' metadata field (if defined)
of the referenced document specified by the 'ref' metadata field.

## Copyright

| Copyright | :copyright: 2024-2026 IOG Singapore, All Rights Reserved |
| --- | --- |
| License | This document is licensed under [CC-BY-4.0] |
| Created | 2024-12-27 |
| Modified | 2026-01-09 |
| Authors | Alex Pozhylenkov <alex.pozhylenkov@iohk.io> |
| | Nathan Bogale <nathan.bogale@iohk.io> |
| | Neil McAuliffe <neil.mcauliffe@iohk.io> |
| | Steven Johnson <steven.johnson@iohk.io> |

### Changelog

#### 0.01 (2025-06-19)

* First Published Version

#### 0.2.2 (2025-12-02)

* Added missing `signers: update: type: "ref"` definition.

[CBOR-TAG-42]: https://github.com/ipld/cid-cbor/
[CBOR-LFD-ENCODING]: https://www.rfc-editor.org/rfc/rfc8949.html#section-4.2.3
[RFC9052-HeaderParameters]: https://www.rfc-editor.org/rfc/rfc8152#section-3.1
[JSON Schema-2020-12]: https://json-schema.org/draft/2020-12
[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
[IPFS-CID]: https://docs.ipfs.tech/concepts/content-addressing/#what-is-a-cid
[RFC9562-V7]: https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-7
