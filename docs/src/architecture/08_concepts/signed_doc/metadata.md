# Metadata Fields

## Metadata Types

The following types of metadata have been defined.
All Metadata fields use one of these types.

### Chain Link

A link to a previous document in a chained sequence.

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "CDDL Specification"

    * [chain.cddl](cddl/chain.cddl)

    ``` cddl
    {{ include_file('./cddl/chain.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

### Collaborators Reference List

A list of collaborators who can participate in drafting and submitting a document

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "CDDL Specification"

    * [collaborators.cddl](cddl/collaborators.cddl)

    ``` cddl
    {{ include_file('./cddl/collaborators.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

### Document Id

A unique document identifier

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "CDDL Specification"

    * [document_id.cddl](cddl/document_id.cddl)

    ``` cddl
    {{ include_file('./cddl/document_id.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

### Document Reference

A document reference identifier

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "CDDL Specification"

    * [document_refs.cddl](cddl/document_refs.cddl)

    ``` cddl
    {{ include_file('./cddl/document_refs.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

### Document Type

A document type identifier

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "CDDL Specification"

    * [document_type.cddl](cddl/document_type.cddl)

    ``` cddl
    {{ include_file('./cddl/document_type.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

### Document Ver

A unique chronological document version

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "CDDL Specification"

    * [document_ver.cddl](cddl/document_ver.cddl)

    ``` cddl
    {{ include_file('./cddl/document_ver.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

### Section Reference

A document section reference identifier

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "CDDL Specification"

    * [section_ref.cddl](cddl/section_ref.cddl)

    ``` cddl
    {{ include_file('./cddl/section_ref.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

### [UUIDv4][RFC9562-V4]

Version 4 formatted [UUID][RFC9562]

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "CDDL Specification"

    * [uuid_v4.cddl](cddl/uuid_v4.cddl)

    ``` cddl
    {{ include_file('./cddl/uuid_v4.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

### [UUIDv7][RFC9562-V7]

Version 7 formatted [UUID][RFC9562]

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "CDDL Specification"

    * [uuid_v7.cddl](cddl/uuid_v7.cddl)

    ``` cddl
    {{ include_file('./cddl/uuid_v7.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

### Version Revocations

A list of all versions of this document which are 'revoked'.

<!-- markdownlint-disable max-one-sentence-per-line MD046 MD013 -->
??? note "CDDL Specification"

    * [revocations.cddl](cddl/revocations.cddl)

    ``` cddl
    {{ include_file('./cddl/revocations.cddl', indent=4) }}
    ```
<!-- markdownlint-enable max-one-sentence-per-line MD046 MD013 -->

## Individual Metadata field definitions

### `type`

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Type](metadata.md#document-type) |
<!-- markdownlint-enable MD033 -->
The document TYPE.

#### `type` Validation

**MUST** be a known document type.

### `id`

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Id](metadata.md#document-id) |
<!-- markdownlint-enable MD033 -->
Document ID, created the first time the document is created.
This must be a properly created [UUIDv7][RFC9562-V7] which contains the
timestamp of when the document was created.

#### `id` Validation

The document ID validation is performed based on timestamp thresholds:

* If `future_threshold` is configured,
the document [`id`](metadata.md#id) cannot be too far in the future from the
current time.
* If `past_threshold` is configured, the document [`id`](metadata.md#id) cannot be too far in the past from the
current time.

### `ver`

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | yes |
| Format | [Document Ver](metadata.md#document-ver) |
<!-- markdownlint-enable MD033 -->
The unique version of the document.
The first version of the document must set [`ver`](metadata.md#ver) == [`id`](metadata.md#id)

[`ver`](metadata.md#ver) represents either:

* when a document changes over time, such as
    with a new version of a particular document that supersedes an
    earlier one.
* when a new document in a sequence of documents is produced.

Because the most common use [`ver`](metadata.md#ver) is a new version of the same document
this is to be assumed unless the document specifies its representing
a sequence of documents.

#### `ver` Validation

1. The document version must always be >= the document ID.
2. IF [`ver`](metadata.md#ver) does not == [`id`](metadata.md#id)
  then a document with [`id`](metadata.md#id) and [`ver`](metadata.md#ver) being equal *MUST* exist.
3. When a document with the same [`id`](metadata.md#id) already exists,
  the new document's [`ver`](metadata.md#ver) must be greater than
  the latest known submitted version for that [`id`](metadata.md#id).
4. When a document with the same [`id`](metadata.md#id) already exists,
  the new document's [`type`](metadata.md#type) must be the same as
  the latest known submitted document's [`type`](metadata.md#type) for that [`id`](metadata.md#id).

### `ref`

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | optional |
| Format | [Document Reference](metadata.md#document-reference) |
| Valid References | [Proposal Form Template](./docs/proposal_form_template.md) |
|  | [Presentation Template](./docs/presentation_template.md) |
|  | [Proposal](./docs/proposal.md) |
|  | [Proposal Comment Form Template](./docs/proposal_comment_form_template.md) |
|  | [Proposal Comment](./docs/proposal_comment.md) |
|  | [Proposal Submission Action](./docs/proposal_submission_action.md) |
|  | [Proposal Moderation Action](./docs/proposal_moderation_action.md) |
|  | [Comment Moderation Action](./docs/comment_moderation_action.md) |
|  | [Brand Parameters](./docs/brand_parameters.md) |
|  | [Brand Parameters Form Template](./docs/brand_parameters_form_template.md) |
|  | [Campaign Parameters](./docs/campaign_parameters.md) |
|  | [Campaign Parameters Form Template](./docs/campaign_parameters_form_template.md) |
|  | [Category Parameters](./docs/category_parameters.md) |
|  | [Category Parameters Form Template](./docs/category_parameters_form_template.md) |
|  | [Contest Parameters](./docs/contest_parameters.md) |
|  | [Contest Parameters Form Template](./docs/contest_parameters_form_template.md) |
|  | [Rep Profile](./docs/rep_profile.md) |
|  | [Rep Profile Form Template](./docs/rep_profile_form_template.md) |
|  | [Rep Nomination](./docs/rep_nomination.md) |
|  | [Rep Nomination Form Template](./docs/rep_nomination_form_template.md) |
|  | [Contest Delegation](./docs/contest_delegation.md) |
|  | [Contest Ballot](./docs/contest_ballot.md) |
|  | [Contest Ballot Checkpoint](./docs/contest_ballot_checkpoint.md) |
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

#### `ref` Validation

The following must be true for a valid reference:

* The Referenced Document **MUST** Exist
* Every value in the `document_locator` must consistently reference the exact same document.
* The `document_id` and `document_ver` **MUST** match the values in the referenced document.
* In the event there are **MULTIPLE** [`ref`](metadata.md#ref) listed, they **MUST** be sorted.

Sorting for each element of [`ref`](metadata.md#ref) follows the same sort order as specified for Map Keys,
as defined by [CBOR Deterministic Encoding][CBOR-LFD-ENCODING] (4.3.2 Length-First Map Key Ordering).

### `template`

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | optional |
| Format | [Document Reference](metadata.md#document-reference) |
| Valid References | [Proposal Form Template](./docs/proposal_form_template.md) |
|  | [Proposal Comment Form Template](./docs/proposal_comment_form_template.md) |
|  | [Brand Parameters Form Template](./docs/brand_parameters_form_template.md) |
|  | [Campaign Parameters Form Template](./docs/campaign_parameters_form_template.md) |
|  | [Category Parameters Form Template](./docs/category_parameters_form_template.md) |
|  | [Contest Parameters Form Template](./docs/contest_parameters_form_template.md) |
|  | [Rep Profile Form Template](./docs/rep_profile_form_template.md) |
|  | [Rep Nomination Form Template](./docs/rep_nomination_form_template.md) |
<!-- markdownlint-enable MD033 -->
Reference to the template used to create and/or validate this document.

#### `template` Validation

In addition to the validation performed for [Document Reference](metadata.md#document-reference) type fields,
The document payload is not valid if it does not validate completely against the referenced template.

### `reply`

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | optional |
| Format | [Document Reference](metadata.md#document-reference) |
| Valid References | [Proposal Comment](./docs/proposal_comment.md) |
<!-- markdownlint-enable MD033 -->
Reference to a Comment document type being referred to.

#### `reply` Validation

In addition to the validation performed for [Document Reference](metadata.md#document-reference) type fields,
The [`ref`](metadata.md#ref) of the [`reply`](metadata.md#reply) document must be the same as
the original comment document.

### `section`

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | optional |
| Format | [Section Reference](metadata.md#section-reference) |
<!-- markdownlint-enable MD033 -->
A Reference to the original document, or the comment being replied to.

#### `section` Validation

For a non-reply this must be a valid section reference into the referenced document.
For a reply, this must be a valid section reference into the comment being replied to.

### `collaborators`

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | optional |
| Format | [Collaborators Reference List](metadata.md#collaborators-reference-list) |
<!-- markdownlint-enable MD033 -->
A list of collaborators who may also publish updates to versions of this document.
This should include all parties who have not signed this document directly.

Every subsequent version can amend the collaborators list.
However, the initial Author can never be removed from being able to
publish a new version of the document.

#### `collaborators` Validation

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

In the event there are **MULTIPLE** [`collaborators`](metadata.md#collaborators) listed, they **MUST** be sorted.

Sorting for each element of [`collaborators`](metadata.md#collaborators) follows the same sort order as specified for Map Keys,
as defined by [CBOR Deterministic Encoding][CBOR-LFD-ENCODING] (4.3.2 Length-First Map Key Ordering).

### `revocations`

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | excluded |

### `parameters`

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | optional |
| Format | [Document Reference](metadata.md#document-reference) |
| Valid References | [Brand Parameters](./docs/brand_parameters.md) |
|  | [Campaign Parameters](./docs/campaign_parameters.md) |
|  | [Category Parameters](./docs/category_parameters.md) |
|  | [Contest Parameters](./docs/contest_parameters.md) |
<!-- markdownlint-enable MD033 -->
A reference to the Parameters Document this document lies under.

#### `parameters` Validation

In addition to the validation performed for [Document Reference](metadata.md#document-reference) type fields:

* Any linked referenced document that includes a [`parameters`](metadata.md#parameters) metadata must match the
[`parameters`](metadata.md#parameters) of the referencing document,
or a parent of those [`parameters`](metadata.md#parameters).

For example, a linked reference to [Contest Parameters](./docs/contest_parameters.md) is transitively a reference to
the Parameters document it references, and each parameters document they reference
until the `Brand` parameters document is reached.

The use case here is for Templates.
The profile template, or proposal templates could be defined at any of these
levels, and as long as they all refer to the same chain of parameters in the
hierarchy they are all valid.

### `conditions`

> **DRAFT STATUS**  
> This metadata field is currently in **DRAFT** status.
> Development should **NOT** begin until this specification is formally released.  
> This specification is subject to change without notice.

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | optional |
| Format | [Document Reference](metadata.md#document-reference) |
| Multiple References | yes |
| Valid References | [Conditions](./docs/conditions.md) |
<!-- markdownlint-enable MD033 -->
An array of references to [Conditions](./docs/conditions.md) documents that define terms and conditions.

The `conditions` metadata field serves two distinct purposes depending on the document type:

1. **On Parameter Documents** (Brand Parameters, Campaign Parameters, Category Parameters, Contest Parameters):
   * Lists the required condition documents for that level of the system hierarchy
   * Specifies which terms users must accept when submitting documents at this level
   * The field is optional - if not present, no conditions are required at that level

2. **On User-Submitted Documents** (Proposals, Proposal Comments, etc.):
   * References all condition documents that the user has accepted
   * Must include ALL conditions required by the parameter hierarchy (Brand → Campaign → Category → Contest)
   * The act of listing these references and signing the document serves as the user's digital signature and acceptance
   * The field is optional when no conditions are required by the parameter hierarchy, but required when conditions are specified

#### `conditions` Validation

**For Parameter Documents:**

* The `conditions` field is optional
* If present, must be an array of valid [Conditions](./docs/conditions.md) document references
* All referenced documents must exist and be of type "Conditions"
* The array must be sorted according to [CBOR Deterministic Encoding][CBOR-LFD-ENCODING] (4.3.2 Length-First Map Key Ordering)
* All array elements must be unique

**For User-Submitted Documents:**

The validation process for user-submitted documents involves transitive collection of required conditions:

1. Extract the [`parameters`](metadata.md#parameters) reference from the user document
2. Follow the parameter chain: Contest → Category → Campaign → Brand
3. Collect all `conditions` arrays from each parameter level in the hierarchy
4. Union all condition references (removing duplicates based on document ID and
   version)
5. Sort the unified list according to [CBOR Deterministic Encoding][CBOR-LFD-ENCODING]
6. Compare the user document's `conditions` array with this unified, sorted list
7. Validation succeeds only if they match exactly

**Validation Rules:**

* The user document's `conditions` array must exactly match the union of all required conditions from the parameter hierarchy
* All referenced Conditions documents must exist and be valid
* All referenced Conditions documents must not be revoked
* The array must be sorted (CBOR deterministic encoding order)
* All array elements must be unique

**Validation Failures:**

The document will be rejected if:

* Missing required conditions (conditions specified in parameter hierarchy but not
  in user document)
* Includes extra conditions (conditions in user document not in parameter
  hierarchy)
* Array is not sorted correctly
* Any referenced condition document doesn't exist or is invalid
* Any referenced condition document is revoked
* Array contains duplicate references

**Edge Cases:**

* **Parameter documents with no `conditions` field**: User documents don't need to include conditions at that level
* **Empty `conditions` arrays**: Valid if no conditions are required by any level in the parameter hierarchy
* **Revoked condition documents**: User documents referencing revoked conditions must be rejected
* **Multiple conditions at same level**: All conditions from a parameter level are required
* **Conditions at multiple levels**: User must accept all conditions from all levels (Brand, Campaign, Category, Contest)

**Example:**

A user submitting a Proposal to a Category must accept:

* All conditions from the Contest Parameters (if any)
* All conditions from the Category Parameters (if any)
* All conditions from the Campaign Parameters (if any)
* All conditions from the Brand Parameters (if any)

The Proposal's `conditions` array must contain the union of all these conditions, sorted and deduplicated.

In the event there are **MULTIPLE** [`conditions`](metadata.md#conditions) listed, they **MUST** be sorted.

Sorting for each element of [`conditions`](metadata.md#conditions) follows the same sort order as specified for Map Keys,
as defined by [CBOR Deterministic Encoding][CBOR-LFD-ENCODING] (4.3.2 Length-First Map Key Ordering).

### `chain`

<!-- markdownlint-disable MD033 -->
| Parameter | Value |
| --- | --- |
| Required | optional |
| Format | [Chain Link](metadata.md#chain-link) |
<!-- markdownlint-enable MD033 -->
An immutable link to the previous document in a chained sequence of documents.
Because ID/Ver only defines values for the current document, and is not intended
by itself to prevent insertion of documents in a sequence, the [`chain`](metadata.md#chain)
metadata allows for the latest document to directly point to its previous iteration.

It also aids in discoverability, where the latest document may be pinned but prior
documents can be discovered automatically by following the chain.

#### `chain` Validation

Chained Documents do not support collaborators.
Any document which is attempted to be published in the sequence
which is *NOT* published by the author of the first document in the
sequence is fraudulent, and to be discarded.

In addition, the chained document *MUST*:

* Not have `collaborators`;
* Have the same [`id`](metadata.md#id) as the document being chained to;
* Have a [`ver`](metadata.md#ver) that is greater than the [`ver`](metadata.md#ver) being chained to;
* Have the same [`type`](metadata.md#type) as the chained document;
* Have [`parameters`](metadata.md#parameters) match;
* Have not be chaining to a document already chained to by another document;
* Have its absolute `height` exactly one more than the `height` of the document being chained to.

IF any of these validations fail, then the entire sequence of documents is INVALID.
Not just the current document.

##### Example of a Valid Chain

<!-- markdownlint-disable MD046 -->
``` mermaid
classDiagram
    direction LR
    class Last {
        type: "=Intermediate.Document Type"
        id: "=Intermediate.Document ID"
        ver: ">Intermediate.Document ID"
        parameters: "=Intermediate.Document Parameters"
        chain.height: -2
        chain.document_ref: "=Intermediate"

        author(Intermediate.Catalyst ID)
    }
    style Last stroke:#060,stroke-width:4px

    class Intermediate {
        type: "=First.Document Type"
        id: "=First.Document ID"
        ver: ">First.Document ID"
        parameters: "=First.Document Parameters"
        chain.height: 1
        chain.document_ref: "=First"

        author(First.Catalyst ID)
    }
    style Intermediate stroke:#060,stroke-width:4px

    class First {
        type: "Document Type"
        id: "Document ID"
        ver: "=Document ID"
        parameters: "Document Parameters"
        chain.height: 0
        chain.document_ref: None

        author(Catalyst ID)
    }
    style First stroke:#060,stroke-width:4px

    Last --|> Intermediate : chains to
    Intermediate --|> First : chains to


```
<!-- markdownlint-enable MD046 -->

##### Example of an Invalid Chain

Either of the two documents being present invalidates the data
in the entire chain,
as they are signed by the author of the chain.

<!-- markdownlint-disable MD046 -->
``` mermaid
classDiagram
    direction LR

    class Last {
        type: "=Intermediate.Document Type"
        id: "=Intermediate.Document ID"
        ver: ">Intermediate.Document ID"
        parameters: "=Intermediate.Document Parameters"
        chain.height: -2
        chain.document_ref: "=Intermediate"

        author(Intermediate.Catalyst ID)
    }
    style Last stroke:#f60,stroke-width:4px

    class Intermediate {
        type: "=First.Document Type"
        id: "=First.Document ID"
        ver: ">First.Document ID"
        parameters: "=First.Document Parameters"
        chain.height: 1
        chain.document_ref: "=First"

        author(First.Catalyst ID)
    }
    style Intermediate stroke:#f60,stroke-width:4px

    class First {
        type: "Document Type"
        id: "Document ID"
        ver: "=Document ID"
        parameters: "Document Parameters"
        chain.height: 0
        chain.document_ref: None

        author(Catalyst ID)
    }
    style First stroke:#f60,stroke-width:4px

    Last --|> Intermediate : chains to
    Intermediate --|> First : chains to

    class Invalid_Chain {
        type: "=First.Document Type"
        id: "=First.Document ID"
        ver: ">Intermediate.Document ID"
        parameters: "=First.Document Parameters"
        chain.height: 1
        chain.document_ref: "=First"

        author(First.Catalyst ID)
    }

    Invalid_Chain --|> First : Invalidly chains to
    style Invalid_Chain fill:#100,stroke:#f00,stroke-width:4px


    class After_Final {
        type: "=Final.Document Type"
        id: "=Final.Document ID"
        ver: ">Final.Document ID"
        parameters: "=Final.Document Parameters"
        chain.height: 3
        chain.document_ref: "=Last"

        author(Last.Catalyst ID)
    }

    After_Final --|> Last : Invalidly chains to
    style After_Final fill:#100,stroke:#f00,stroke-width:4px

```
<!-- markdownlint-enable MD046 -->

##### Example of a Fraudulent Chain Document

The invalid document does not invalidate the chain,
as its not signed by the author of the chained documents.

<!-- markdownlint-disable MD046 -->
``` mermaid
classDiagram
    direction LR
    class Last {
        type: "=Intermediate.Document Type"
        id: "=Intermediate.Document ID"
        ver: ">Intermediate.Document ID"
        parameters: "=Intermediate.Document Parameters"
        chain.height: -2
        chain.document_ref: "=Intermediate"

        author(Intermediate.Catalyst ID)
    }
    style Last stroke:#060,stroke-width:4px

    class Intermediate {
        type: "=First.Document Type"
        id: "=First.Document ID"
        ver: ">First.Document ID"
        parameters: "=First.Document Parameters"
        chain.height: 1
        chain.document_ref: "=First"

        author(First.Catalyst ID)
    }
    style Intermediate stroke:#060,stroke-width:4px

    class First {
        type: "Document Type"
        id: "Document ID"
        ver: "=Document ID"
        parameters: "Document Parameters"
        chain.height: 0
        chain.document_ref: None

        author(Catalyst ID)
    }
    style First stroke:#060,stroke-width:4px

    Last --|> Intermediate : chains to
    Intermediate --|> First : chains to

    class Rejected {
        type: "=First.Document Type"
        id: "=First.Document ID"
        ver: ">Intermediate.Document ID"
        parameters: "=Intermediate.Document Parameters"
        chain.height: 1
        chain.document_ref: "=First"

        author(Other.Catalyst ID)
    }

    Rejected --|> Intermediate : Invalidly chains to
    style Rejected fill:#100,stroke:#f00,stroke-width:4px

```
<!-- markdownlint-enable MD046 -->

## Copyright

| Copyright | :copyright: 2024-2025 IOG Singapore, All Rights Reserved |
| --- | --- |
| License | This document is licensed under [CC-BY-4.0] |
| Created | 2024-12-27 |
| Modified | 2025-12-02 |
| Authors | Alex Pozhylenkov <alex.pozhylenkov@iohk.io> |
| | Nathan Bogale <nathan.bogale@iohk.io> |
| | Neil McAuliffe <neil.mcauliffe@iohk.io> |
| | Steven Johnson <steven.johnson@iohk.io> |

[CBOR-TAG-42]: https://github.com/ipld/cid-cbor/
[CBOR-LFD-ENCODING]: https://www.rfc-editor.org/rfc/rfc8949.html#section-4.2.3
[CC-BY-4.0]: https://creativecommons.org/licenses/by/4.0/legalcode
[IPFS-CID]: https://docs.ipfs.tech/concepts/content-addressing/#what-is-a-cid
[RFC9562-V4]: https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-4
[RFC9562-V7]: https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-7
[RFC9562]: https://www.rfc-editor.org/rfc/rfc9562.html
