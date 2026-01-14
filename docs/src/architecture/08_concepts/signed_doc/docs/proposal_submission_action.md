# Proposal Submission Action

## Description

Proposal Submission Action

A Proposal Submission Action is a document which can attempt to either submit a
particular version of a proposal into a campaign, or withdraw it.

The last action on the document is the action which takes effect at the deadline.

For multiple collaborators, multiple submission actions can be posted independently.
How those submissions are counted is controlled by a parameter defined in the relevant
Brand/Campaign/Category parameters (parameter name TBD):

* If configured for unanimous collaboration, actions for a proposal version do not take
  effect until the author and **all** listed collaborators have posted equivalent actions.
* If configured for opt-in collaboration (the default, and the behavior when the parameter
  is absent), the author must submit `final`, and collaborators are only counted on the
  submission when they submit `final` for that version; other listed collaborators are not
  required to post equivalent actions.

For example, three collaborators Alice/Bob/Claire can each post one submission action
for the same document.
Under the unanimous configuration, unless they all submit the same version of the proposal
the proposal will not be seen as submitted.
Under the opt-in configuration, the author must submit `final` and whichever collaborators
also submit `final` for that version are counted as collaborators on that submission.

The required set of signers for a submitted proposal version is:

* The original author of the proposal (the signer of the first version where
  [`id`](../metadata.md#id) == [`ver`](../metadata.md#ver)); and
* A collaborator set derived from the configuration described above.

When collaborator unanimity is configured, that set is every collaborator listed in
[`collaborators`](../metadata.md#collaborators) on the **exact** version of the proposal
referenced by [`ref`](../metadata.md#ref).
When opt-in is configured (the default/fallback), only collaborators who submit `final`
for the referenced version are included as collaborators on that submission.
In all modes, a proposal is only final if the author has submitted `final`.

They may jointly sign a single Proposal Submission Action, or may submit multiple independent
Submission actions for the same document (which avoids the need for multi-sig coordination).

The payload is a fixed format.

<!-- markdownlint-disable max-one-sentence-per-line -->

```graphviz dot proposal_submission_action.dot.svg

{{ include_file('./../diagrams/proposal_submission_action.dot', indent=4) }}
```

<!-- markdownlint-enable max-one-sentence-per-line -->

### Validation

No validation is required beyond as defined by:

* [metadata](#metadata)
* [payload](#payload)
* [signers](#signers)

### Business Logic

#### Front End

A proposal with [`collaborators`](../metadata.md#collaborators) will not be shown as having a confirmed collaborator,
unless there exists a `draft` or `final` proposal submission from that collaborator.

Any document that lists a collaborator should be highlighted to that collaborator so
they can take appropriate action, such as:

* Confirm they are a collaborator by submitting this document as `draft`
* Agree to being a collaborator on the final submission by submitting this document as `final`
* Hide themselves from the collaborators list but do not remove themselves by submitting `hide`
* Remove themselves permanently as a collaborator by publishing a new version with them removed.

To eliminate the necessity for collaborators to accept collaboration on every version,
they will be considered as agreeing to be a collaborator on **any** version of the proposal that
lists them in [`collaborators`](../metadata.md#collaborators), if their latest submission for that
proposal is `draft` or `final`.

If their latest submission on a proposal is `hide` they **MUST** be considered to not have agreed
to be a collaborator for **any** version of that proposal (past, present, or future) until they
submit a new `draft` or `final` action.

Whether collaborator submissions are required for finalization follows the
Brand/Campaign/Category parameter described above.
If the parameter is absent, only collaborators who submit `final` for the referenced version are
counted as collaborators on that submission; listed collaborators who do not submit `final` are
not required for the proposal to be considered final.

*NOTE* `final` status ONLY applies to the exactly referenced document and version.

#### Back End

A Submitted proposal with collaborators *MUST* have, by the configured deadline:

* A `final` submission from the author; and
* Collaborator submissions as required by the Brand/Campaign/Category parameter that controls
  collaborator finalization (parameter name TBD):
    * If set to unanimous, a `final` submission from every collaborator listed in
      [`collaborators`](../metadata.md#collaborators) on the version of the proposal
      referenced by [`ref`](../metadata.md#ref); or
    * If set to opt-in (the default, and the behavior when the parameter is absent), only
      collaborators whose latest submission for that proposal version is `final` are included as
      collaborators on the submission.
* No required signer (author or any collaborator counted under the configured mode) whose latest
  submission for that proposal is `hide`.

If the author has not submitted a `final` submission for that proposal version by the deadline,
or if any collaborator required by the configured mode has not submitted a `final` submission, or
if any required signer’s latest submission is `hide`,
then the proposal is not considered `final` and will not be considered in the
category it was being submitted to.

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
| Type | 5e60e623-ad02-4a1b-a1ac-406db978ee48 |
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
| Valid References | [Proposal](proposal.md) |
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

### [`parameters`](../metadata.md#parameters)

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Reference](../metadata.md#document-reference) |
| Valid References | [Brand Parameters](brand_parameters.md) |
|  | [Campaign Parameters](campaign_parameters.md) |
|  | [Category Parameters](category_parameters.md) |
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

The kind of action is controlled by this payload.
The Payload is a [JSON][RFC8259] Document, and must conform to this schema.

States:

* `final` : Signer marks this proposal version as their final submission.
            Whether all collaborators must also submit `final` is controlled by the
            Brand/Campaign/Category parameter noted above.
            If the parameter is absent, only collaborators who submit `final` for the referenced
            version are counted as collaborators on the submission, but the author’s `final` is
            always required.
* `draft` : Reverses the previous `final` state for a signer and accepts collaborator status to a document.
* `hide`  : Requests the proposal be hidden (not final, but a hidden draft).
         `hide` is only actioned if sent by the author,
         for a collaborator it identified that they do not wish to be listed as a `collaborator`.

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
      "title": "Proposal Submission Action Payload Schema",
      "description": "Structure of the payload of a Proposal Submission Action.",
      "$defs": {
        "action": {
          "description": "The action being performed on the Proposal.",
          "enum": [
            "final",
            "draft",
            "hide"
          ],
          "type": "string"
        }
      },
      "type": "object",
      "properties": {
        "action": {
          "$ref": "#/$defs/action"
        }
      },
      "additionalProperties": false,
      "required": [
        "action"
      ],
      "x-changelog": {
        "2025-03-01": [
          "First Version Created."
        ]
      }
    }
    ```
<!-- markdownlint-enable MD013 MD046 max-one-sentence-per-line -->

### Examples
<!-- markdownlint-disable MD013 MD046 max-one-sentence-per-line -->
??? example "Example: Final Proposal Submission"

    This document indicates the linked proposal is final and requested to proceed for further consideration.

    ```json
    {
      "action": "final"
    }
    ```

<!-- markdownlint-enable MD013 MD046 max-one-sentence-per-line -->
<!-- markdownlint-disable MD013 MD046 max-one-sentence-per-line -->
??? example "Example: Draft Proposal Submission"

    This document indicates the linked proposal is no longer final and should not proceed for further consideration.
    It is also used by collaborators to accept that they are a collaborator on a document.

    ```json
    {
      "action": "draft"
    }
    ```

<!-- markdownlint-enable MD013 MD046 max-one-sentence-per-line -->
<!-- markdownlint-disable MD013 MD046 max-one-sentence-per-line -->
??? example "Example: Hidden Proposal Submission"

    If submitted by the proposal author the document is hidden, it is still public but not shown as
    a proposal being drafted.
    If submitted by a collaborator, that collaborator is declaring they do not wish to be listed as
    a collaborator on the proposal.

    ```json
    {
      "action": "hide"
    }
    ```

<!-- markdownlint-enable MD013 MD046 max-one-sentence-per-line -->

## Signers

The following User roles may sign documents of this type:

* Proposer

Updates are allowed by the original author OR from the 'collaborators' metadata field (if defined)
of the referenced document specified by the 'ref' metadata field.

## Copyright

| Copyright | :copyright: 2024-2026 IOG Singapore, All Rights Reserved |
| --- | --- |
| License | This document is licensed under [CC-BY-4.0] |
| Created | 2024-12-27 |
| Modified | 2026-01-13 |
| Authors | Alex Pozhylenkov <alex.pozhylenkov@iohk.io> |
| | Nathan Bogale <nathan.bogale@iohk.io> |
| | Neil McAuliffe <neil.mcauliffe@iohk.io> |
| | Steven Johnson <steven.johnson@iohk.io> |

### Changelog

#### 0.01 (2025-04-04)

* First Published Version

#### 0.03 (2025-05-05)

* Use generalized parameters.

[CBOR-TAG-42]: https://github.com/ipld/cid-cbor/
[CBOR-LFD-ENCODING]: https://www.rfc-editor.org/rfc/rfc8949.html#section-4.2.3
[RFC9052-HeaderParameters]: https://www.rfc-editor.org/rfc/rfc8152#section-3.1
[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
[IPFS-CID]: https://docs.ipfs.tech/concepts/content-addressing/#what-is-a-cid
[RFC9562-V7]: https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-7
[RFC8259]: https://www.rfc-editor.org/rfc/rfc8259.html
