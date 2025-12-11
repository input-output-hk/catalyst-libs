// Signed Document Definitions
//
// Metadata Types and Constraints
@extern(embed)
package signed_docs

import (
	"list"
	"github.com/input-output-hk/catalyst-libs/specs/generic:optional"
	"github.com/input-output-hk/catalyst-libs/specs/signed_doc_types"
	CDDL "github.com/input-output-hk/catalyst-libs/specs/cddl"
)

// Metadata Formats.
// format_name : cddl definition
#metadataFormats: {
	[string]: {
		description: string
		cddl:        CDDL.#cddlTypesConstraint
	}
}

metadata: formats: #metadataFormats
metadata: formats: {
	"Document Reference": {
		description: "A document reference identifier"
		cddl:        "document_refs"
	}
	UUIDv7: {
		description: "Version 7 formatted UUID"
		cddl:        "uuid_v7"
	}
	UUIDv4: {
		description: "Version 4 formatted UUID"
		cddl:        "uuid_v4"
	}
	"Document Type": {
		description: "A document type identifier"
		cddl:        "document_type"
	}
	"Document Id": {
		description: "A unique document identifier"
		cddl:        "document_id"
	}
	"Document Ver": {
		description: "A unique chronological document version"
		cddl:        "document_ver"
	}

	"Section Reference": {
		description: "A document section reference identifier"
		cddl:        "section_ref"
	}
	"Collaborators Reference List": {
		description: "A list of collaborators who can participate in drafting and submitting a document"
		cddl:        "collaborators"
	}
	"Version Revocations": {
		description: "A list of all versions of this document which are 'revoked'."
		cddl:        "revocations"
	}
	"Chain Link": {
		description: "A link to a previous document in a chained sequence."
		cddl:        "chain"
	}
}

// Types of a Metadata Fields
#metadataTypes: [
	for k, _ in metadata.formats {k},
]

// Constraint of Types of Metadata Fields
#metadataTypesConstraint: or(#metadataTypes)

// Canonical List of all valid metadata names
// Must be listed in preferred order
_metadataNames: list.UniqueItems
_metadataNames: [
	"type",
	"id",
	"ver",
	"ref",
	"template",
	"reply",
	"section",
	"collaborators",
	"revocations",
	"parameters",
	"conditions",
	"chain",
]

_allMetadataNames: or([
	for k in _metadataNames {k},
])
// Definition of a metadata field.
#metadataField: {
	// Is the field required to be present.
	required: optional.#field_default_excluded

	// Format of the field.
	format: #metadataTypesConstraint | *#metadataTypes[0]

	if format == "Document Reference" && required != "excluded" {
		type: signed_doc_types.#allDocNames | [...signed_doc_types.#allDocNames]
		multiple: bool | *false
		linked_refs: [..._allMetadataNames] | *null
	}

	// Markdown description of the field.
	description: string
	// Optional notes about validating the field.
	validation: string
}

// Metadata fields that are optional
#metadataStruct: {
	[_allMetadataNames]: #metadataField
}

#metadata: #metadataStruct
#metadata: {
	// Document Type
	type: {
		required:    "yes"
		format:      "Document Type"
		description: "The document TYPE."
		validation: """
			**MUST** be a known document type.
			"""
	}
	// Document ID
	id: {
		required: "yes"
		format:   "Document Id"
		description: """
			Document ID, created the first time the document is created.
			This must be a properly created UUIDv7 which contains the
			timestamp of when the document was created.
			"""
		validation: """
			The document ID validation is performed based on timestamp thresholds:
			
			* If `future_threshold` is configured,
			the document `id` cannot be too far in the future from the
			current time.
			* If `past_threshold` is configured, the document `id` cannot be too far in the past from the
			current time.
			"""
	}

	ref: {
		description: """
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

			* `cid` : A CBOR Encoded IPLD Content Identifier ( AKA an IPFS CID ).
			* Others may be added when further storage mechanisms are defined.

			The document location does not guarantee that the document is actually stored.
			It only defines that if it were stored, this is the identifier
			that is required to retrieve it.
			Therefore it is required that Document References
			are unique and reproducible, given a documents contents.
			"""
		validation: """
			The following must be true for a valid reference:

			* The Referenced Document **MUST** Exist
			* Every value in the `document_locator` must consistently reference the exact same document.
			* The `document_id` and `document_ver` **MUST** match the values in the referenced document.
			* In the event there are **MULTIPLE** `ref` listed, they **MUST** be sorted.

			Sorting for each element of `ref` follows the same sort order as specified for Map Keys, 
			as defined by CBOR Deterministic Encoding (4.3.2 Length-First Map Key Ordering).
			"""
	}

	template: {
		description: "Reference to the template used to create and/or validate this document."
		validation: """
			In addition to the validation performed for `Document Reference` type fields,
			The document payload is not valid if it does not validate completely against the referenced template.
			"""
	}

	reply: {
		description: """
			Reference to a Comment document type being referred to.
			"""
		validation: """
			In addition to the validation performed for `Document Reference` type fields,
			The `ref` of the `reply` document must be the same as
			the original comment document.
			"""
	}

	section: {
		format: "Section Reference"
		description: """
			A Reference to the original document, or the comment being replied to.
			"""
		validation: """
			For a non-reply this must be a valid section reference into the referenced document.
			For a reply, this must be a valid section reference into the comment being replied to.
			"""
	}

	collaborators: {
		format: "Collaborators Reference List"
		description: """
			A list of collaborators who may also publish updates to versions of this document.
			This should include all parties who have not signed this document directly.

			Every subsequent version can amend the collaborators list.
			However, the initial Author can never be removed from being able to
			publish a new version of the document.
			"""
		validation: """
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

			In the event there are **MULTIPLE** `collaborators` listed, they **MUST** be sorted.

			Sorting for each element of `collaborators` follows the same sort order as specified for Map Keys, 
			as defined by CBOR Deterministic Encoding (4.3.2 Length-First Map Key Ordering).
			"""
	}

	revocations: {
		format: "Version Revocations"
		description: """
			A document may include a list of any prior versions which are considered to be revoked.
			Only the revocation list in the latest version of the document applies.
			Revoked documents are flagged as no longer valid, and should not be displayed.
			As a special case, if the revocations are set to `true` then all versions of the document
			are revoked, including the latest document.

			In this case, when the latest document is revoked, the payload may be `nil`.
			Any older document that has `revocations` set to `true` is always to be filtered
			and its payload is to be assumed to be invalid.

			This allows for an entire document and any/all published versions to be revoked.
			A new version of the document that is published after this, may reinstate prior
			document versions, by not listing them as revoked.
			However, any document where revocations was set `true` can never be reinstated.
			"""
		validation: """
			If the field is `true` the payload may be absent or invalid.
			Such documents may never be submitted.
			"""
	}

	conditions: {
		format: "Document Reference"
		description: """
			An array of references to Conditions documents that define terms and conditions.

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
			"""
		validation: """
			**For Parameter Documents:**

			* The `conditions` field is optional
			* If present, must be an array of valid Conditions document references
			* All referenced documents must exist and be of type "Conditions"
			* The array must be sorted according to CBOR Deterministic Encoding (4.3.2 Length-First Map Key Ordering)
			* All array elements must be unique

			**For User-Submitted Documents:**

			The validation process for user-submitted documents involves transitive collection of required conditions:

			1. Extract the `parameters` reference from the user document
			2. Starting from the referenced parameters document, follow the parent chain upward to Brand:
			   * If the document references Contest Parameters, determine its parent by examining the Contest Parameters document's `parameters` metadata field, then follow the appropriate path:
			     - If Contest's parent is Brand: Contest → Brand
			     - If Contest's parent is Campaign: Contest → Campaign → Brand
			     - If Contest's parent is Category: Contest → Category → Campaign → Brand
			   * If the document references Category Parameters, follow: Category → Campaign → Brand
			   * If the document references Campaign Parameters, follow: Campaign → Brand
			   * If the document references Brand Parameters, only Brand is included

			   Note: Contest Parameters can anchor to Brand, Campaign, or Category (as specified in the Contest Parameters document's `parameters` field). The validation must examine the actual Contest Parameters document to determine which parent chain applies. This is not a fixed hierarchy - the path depends on where the specific Contest is anchored.
			3. Collect all `conditions` arrays from each parameter level encountered in the upward chain
			4. Union all condition references (removing duplicates based on document ID and version)
			5. Sort the unified list according to CBOR Deterministic Encoding
			6. Compare the user document's `conditions` array with this unified, sorted list
			7. Validation succeeds only if they match exactly

			Important: The chain traversal always moves UPWARD from the document's referenced parameters document to Brand. Contest Parameters are only included if the document directly references them. The chain order depends on where the document is anchored in the hierarchy.

			**Validation Rules:**

			* The user document's `conditions` array must exactly match the union of all required conditions from the parameter hierarchy
			* All referenced Conditions documents must exist and be valid
			* All referenced Conditions documents must not be revoked
			* The array must be sorted (CBOR deterministic encoding order)
			* All array elements must be unique

			**Validation Failures:**

			The document will be rejected if:
			* Missing required conditions (conditions specified in parameter hierarchy but not in user document)
			* Includes extra conditions (conditions in user document not in parameter hierarchy)
			* Array is not sorted correctly
			* Any referenced condition document doesn't exist or is invalid
			* Any referenced condition document is revoked
			* Array contains duplicate references
			"""
	}

}

// Note: we make all normally excluded fields optional at the global level, because they are globally optional
metadata: headers: #metadata
metadata: headers: {
	ref: required:           "optional"
	ref: type:               signed_doc_types.allDocNames
	template: required:      "optional"
	template: type:          signed_doc_types.templateDocNamesList
	reply: required:         "optional"
	reply: type:             signed_doc_types.commentDocNamesList
	section: required:       "optional"
	collaborators: required: "optional"
	conditions: {
		required: "optional"
		format:   "Document Reference"
		type:     ["Conditions"]
		multiple: true
	}
}

// Preferred display order
// If metadata field not listed, then list them after the explicit ones, in alphabetical order.
metadata: order: list.UniqueItems
metadata: order: [..._allMetadataNames] & _metadataNames
