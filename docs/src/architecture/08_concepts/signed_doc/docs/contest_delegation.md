# Contest Delegation

## Description

Delegation by a Registered User to a Representative for
a contest.

This delegation allows votes cast by the Representative
to use the voting power of the delegating User, in addition
to their own personal voting power and that of all other Users
who delegate to the same Representative.

Delegation is for a specific Contest.
Multiple Delegations must be published if there are multiple
Contests within a Brand/Campaign or Category.

This is because different Contests may have different rules.
And not all Representatives will choose to (or be able to) nominate
for every Contest.

A Representative ***MAY NOT*** delegate to a different Representative
for any contest they have nominated for.
They ***MAY*** however nominate a Representative in any contest they
have not nominated for.

A Representative is NOT required to delegate to themselves in a contest they are nominated for,
and in fact, any self-delegation is invalid and ignored.
A Representative has an implicit 100% voting power delegation to themselves in any contest
they are nominated.
The MAY not vote personally, and if they do, that vote will have Zero (0) voting power.
100% of their voting power is assigned to their delegate vote and can not be split in any way.

A voter MAY choose multiple delegates for a contest, in this case they are listed in priority
order from highest priority to lowest.
Priority only affects two aspects of the delegation.

1. Any residual voting power after it is split among all delegates is given to the highest
   priority delegate (first).
2. If there is not enough voting power to distribute, then its distributed from highest
   priority to lowest.
   This may mean that low priority delegates get zero voting power.

An example:  If a Voter has 100 raw voting power, after quadratic scaling, they have 10.
If they delegated to 15 delegates equally, then only the first 10 would get 1 voting power
each.
Voting power is not fractionally assigned.

The payload MAY contain a [json][RFC8259] document which consists of a single array which can adjust
the ratio of the delegation.
Voting power is divided based on the weight of a single delegate over the sum of all
weights of all delegates.
This is performed with integer division.
As a special condition, 0 or any negative value is equivalent to a weight of 1.
As explained above, if there is not enough voting power to distribute, low priority reps
will receive 0 voting power from the delegation.
And if there is any residual after integer division its applied to the representative
with the highest priority.

<!-- markdownlint-disable max-one-sentence-per-line -->

```graphviz dot contest_delegation.dot.svg

{{ include_file('./../diagrams/contest_delegation.dot', indent=4) }}
```

<!-- markdownlint-enable max-one-sentence-per-line -->

### Validation

* The [`parameters`](../metadata.md#parameters) metadata *MUST* point to the same Contest as the
    Nomination of the Representative.
* The 'ref' metadata field MUST point to a valid 'Representative Nomination'.
    IF there are multiple representatives, then any which are not pointing
    to a valid `Representative Nomination` are excluded.
    The nomination is only invalid if ALL references `Representative Nomination`
    references are invalid.
    This is to prevent a Representative changing their nomination invalidating a
    delegation with multiple representatives.
* The payload MUST be nil.

A Representative *MUST* Delegate to their latest Nomination for a Category,
otherwise their Nomination is invalid.

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

A Voter may withdraw their Delegation by revoking all delegation documents.
[`revocations`](../metadata.md#revocations) must be set to `true` to withdraw a delegation, OR
a later contest delegation may change the delegated representative without
first revoking the prior delegation, as only the latest delegation is
considered.

### Business Logic

#### Front End

* Allows a voter to select a Representative from a list of eligible candidates for a category.
* The voter signs this document to confirm their delegation choice.

#### Back End

* Verifies that the voter and Representative are valid and registered for the category.
* Records the delegation of voting power from the voter to the Representative.

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
| Type | 764f17fb-cc50-4979-b14a-b213dbac5994 |
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
| Multiple References | True |
| Valid References | [Rep Nomination](rep_nomination.md) |
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
| Linked Reference Metadata | [`ref`](#ref) |
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

* The Document referenced by [`ref`](../metadata.md#ref)
    * MUST contain [`parameters`](../metadata.md#parameters) metadata; AND
    * MUST match the referencing documents [`parameters`](../metadata.md#parameters) value.

## Payload

The Payload is a [JSON][RFC8259] Document, and must conform to this schema.

It consists of an array which defines the weights to be applied to the chosen delegations.

Each valid delegate gets the matching weight from this array.
The total voting power is split proportionally based on these weights over the
valid drep nominations.

### Schema

<!-- markdownlint-disable MD013 MD046 max-one-sentence-per-line -->
??? abstract "Schema: Payload [JSON][RFC8259] Schema"

    ```json
    {
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "maintainers": [
        {
          "name": "Catalyst Team",
          "url": "https://projectcatalyst.io/"
        }
      ],
      "title": "Contest Delegation Schema",
      "description": "Structure of the payload of a Contest Delegation.",
      "type": "object",
      "properties": {
        "weights": {
          "description": "List of weights to apply to each delegate.\nThis list is in the same order as the delegate references.\nIf there are fewer entries than delegates, then the missing weights are set to `1`.\nIf there are more weights, then the extra weights are ignored.  If the payload is missing, OR the array is empty, then the weights assigned is `1`.",
          "items": {
            "exclusiveMinimum": 0,
            "type": "integer"
          },
          "minItems": 0,
          "type": "array"
        }
      },
      "additionalProperties": false,
      "required": [
        "weights"
      ],
      "x-changelog": {
        "2025-03-01": [
          "First Version Created."
        ]
      }
    }
    ```
<!-- markdownlint-enable MD013 MD046 max-one-sentence-per-line -->

### Example
<!-- markdownlint-disable MD013 MD046 max-one-sentence-per-line -->
??? example "Example: Three Delegation Weights"

    If there are only 1 delegation, then the weights do not matter.
    If there are two, then the first delegate has a weight of 10/30, and the second has 20/30.
    If there are 5, then the weights are: `[10,20,30,1,1]`

    ```json
    {
      "weights": [
        10,
        20,
        30
      ]
    }
    ```

<!-- markdownlint-enable MD013 MD046 max-one-sentence-per-line -->

## Signers

The following User roles may sign documents of this type:

* Registered

Only the original author can update and sign a new version of documents.

## Copyright

| Copyright | :copyright: 2024-2025 IOG Singapore, All Rights Reserved |
| --- | --- |
| License | This document is licensed under [CC-BY-4.0] |
| Created | 2024-12-27 |
| Modified | 2025-12-15 |
| Authors | Alex Pozhylenkov <alex.pozhylenkov@iohk.io> |
| | Nathan Bogale <nathan.bogale@iohk.io> |
| | Neil McAuliffe <neil.mcauliffe@iohk.io> |
| | Steven Johnson <steven.johnson@iohk.io> |

### Changelog

#### 0.01 (2025-06-19)

* First Published Version

#### 0.1.2 (2025-09-04)

* Allow Multi Delegation

[CBOR-TAG-42]: https://github.com/ipld/cid-cbor/
[CBOR-LFD-ENCODING]: https://www.rfc-editor.org/rfc/rfc8949.html#section-4.2.3
[RFC9052-HeaderParameters]: https://www.rfc-editor.org/rfc/rfc8152#section-3.1
[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
[IPFS-CID]: https://docs.ipfs.tech/concepts/content-addressing/#what-is-a-cid
[RFC9562-V7]: https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-7
[RFC8259]: https://www.rfc-editor.org/rfc/rfc8259.html
