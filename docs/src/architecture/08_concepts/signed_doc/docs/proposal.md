# Proposal

## Description

A Proposal is a document which describes a proposed solution or project to
address the criteria of a category within a campaign.

The proposal itself is a draft document, it is not submitted for consideration
unless a [Proposal Submission Action](proposal_submission_action.md) is submitted which references it.

Proposals themselves are intentionally general, however they may be
linked to a brand/campaign or category via the template used by the proposal.

The payload of a proposal is controlled by its template.

<!-- markdownlint-disable max-one-sentence-per-line -->

```graphviz dot proposal.dot.svg

{{ include_file('./../diagrams/proposal.dot', indent=4) }}
```

<!-- markdownlint-enable max-one-sentence-per-line -->

### Validation

The first version of a Proposal *MUST* be signed by the original author.
It may optionally be co-signed by any of the listed [`collaborators`](../metadata.md#collaborators).
It may not be signed by anyone else.

Subsequent Versions can be signed/co-signed by either the Original Author of the first version,
OR any of the listed [`collaborators`](../metadata.md#collaborators) in the immediately previous version.
This allows any collaborator to update the next version of a document, provided they are still a collaborator.
It is valid for a proposal to be signed by a collaborator
who is no longer listed as in the [`collaborators`](../metadata.md#collaborators)
of the document they are signing, provided they are listed as a collaborator in the immediately previous document version.
This allows for a collaborator to make an update to the document which removes themselves
from the [`collaborators`](../metadata.md#collaborators) list.

All versions of the document are owned by the original author.
The Author can not be changed by any document revision.

Any Proposal that lists a collaborator is an invitation for that collaborator to participate in the proposal.
They are considered to have accepted that invitation for **all** versions of the proposal that
list them as a collaborator where their latest
[Proposal Submission Action](proposal_submission_action.md) for that proposal has an `action` of
`draft` or `final`.

If a collaborator’s latest [Proposal Submission Action](proposal_submission_action.md) for the
proposal has an `action` of `hide`, they **MUST** be treated as not having agreed to collaborate
for **any** version of that proposal (past, present, or future) until they later submit `draft`
or `final` again.

The requirement for collaborator submissions when finalizing a proposal is controlled by a
Brand/Campaign/Category parameter (name TBD).
When configured for unanimous collaboration,
every collaborator listed on the submitted version **MUST** also publish a `final`
[Proposal Submission Action](proposal_submission_action.md) alongside the author.
When configured for opt-in collaboration (the default, and the behavior when the parameter is
absent), only collaborators who submit `final` for the referenced version are included as
collaborators on that submission; collaborators who do not submit `final` are not treated as
collaborators for that submission.
In all cases, a proposal cannot be final unless the original author has submitted `final`.

The `final` proposal itself may be signed by one or more Collaborators and/or the original Author.
The `final` proposal must never be signed by anyone else.

### Business Logic

#### Front End

As validity of the documents is currently enforced by the backend,
the front end does not need to validate the document has been signed
correctly.
It may do so, but it is not required.

#### Back End

Before accepting a new proposal to be published, the backend will ensure:

* The document has been signed by a valid author or collaborator.
* That the signer of the document was a registered proposer
* That the document was signed with their proposers key
* That all listed [`collaborators`](../metadata.md#collaborators) are registered as proposers.
* That the document has been signed validly according to the [validation](#validation) rules.

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
| Type | 7808d2ba-d511-40af-84e8-c0d1625fdfdc |
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

### [`template`](../metadata.md#template)

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Reference](../metadata.md#document-reference) |
| Valid References | [Proposal Form Template](proposal_form_template.md) |
<!-- markdownlint-enable MD033 -->
Reference to the template used to create and/or validate this document.

#### [`template`](../metadata.md#template) Validation

In addition to the validation performed for [Document Reference](../metadata.md#document-reference) type fields,
The document payload is not valid if it does not validate completely against the referenced template.

### [`collaborators`](../metadata.md#collaborators)

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | optional |
| Format | [Collaborators Reference List](../metadata.md#collaborators-reference-list) |
<!-- markdownlint-enable MD033 -->
A list of collaborators who may also publish updates to versions of this document.
This should include all parties who have not signed this document directly.

Every subsequent version can amend the collaborators list.
However, the initial Author can never be removed from being able to
publish a new version of the document.

#### [`collaborators`](../metadata.md#collaborators) Validation

This list does not imply these collaborators have consented to collaborate, only that the author/s
are permitting these potential collaborators to participate in the drafting and submission process.
How collaborators are counted on a final submission is determined by a parameter defined at the
Brand/Campaign/Category level (parameter name TBD).
Depending on that configuration:

* All listed collaborators may be required to submit a `final` Submission Action in addition
  to the author; **OR**
* Only collaborators who submit a `final` Submission Action for the referenced version are
  included as collaborators on that submission.

If the parameter is not present, default to the latter mode (only final-signing collaborators are
included).
In all modes a document is only considered final when the original author has submitted `final`.

In the event there are **MULTIPLE** [`collaborators`](../metadata.md#collaborators) listed, they **MUST** be sorted.

Sorting for each element of [`collaborators`](../metadata.md#collaborators) follows the same sort order as specified for Map Keys,
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
| Valid References | [Brand Parameters](brand_parameters.md) |
|  | [Campaign Parameters](campaign_parameters.md) |
|  | [Category Parameters](category_parameters.md) |
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

Proposal Document drafted for submission to a category of a campaign.

Must be valid according to the schema contained within the
[Document Reference](../metadata.md#document-reference) from the [`template`](../metadata.md#template) metadata.

## Signers

The following User roles may sign documents of this type:

* Proposer

Updates are allowed by the original author and from the 'collaborators' metadata field
of the previous submitted document's version.

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

[CBOR-LFD-ENCODING]: https://www.rfc-editor.org/rfc/rfc8949.html#section-4.2.3
[RFC9052-HeaderParameters]: https://www.rfc-editor.org/rfc/rfc8152#section-3.1
[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
[RFC9562-V7]: https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-7
