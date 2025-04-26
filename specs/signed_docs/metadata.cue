// Signed Document Definitions
// 
// Metadata Types and Constraints
package signed_docs

import (
	"list"
	"github.com/input-output-hk/catalyst-libs/specs/generic:optional"

)

// Metadata Formats.
// format_name : cddl definition
#metadataFormats: {
	[string]: {
		description: string
		cddl:        #cddlTypesConstraint
	}
}

metadataFormats: #metadataFormats & {
	"Document Reference": {
		description: "A document reference identifier"
		cddl:        "document_ref"
	}
	"UUIDv7": {
		description: "Version 7 formatted UUID"
		cddl:        "uuid_v7"
	}
	"UUIDv4": {
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
}

// Types of a Metadata Fields
#metadataTypes: [
	for k, _ in metadataFormats {k},
]

// Constraint of Types of Metadata Fields
#metadataTypesConstraint: or(#metadataTypes)

// Canonical List of all valid metadata names
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
	"brand_id",
	"campaign_id",
	"election_id",
	"category_id",
]

_allMetadataNames: or([
	for k in _metadataNames {k},
])
// Definition of a metadata field.
#metadataField: {
	// Is the field required to be present.
	required: optional.#field

	// Format of the field.
	format: #metadataTypesConstraint | *#metadataTypes[0]

	if format == "Document Reference" && required != "excluded" {
		type: #DocumentName | [...#DocumentName]
		multiple: bool | *false
		linked_refs: [..._allMetadataNames] | *null
	}

	// Markdown description of the field.
	description: string | *""
	// Optional notes about validating the field.
	validation: string | *null

	// Is the field exclusive of another field (ie can not exist with that other field in the same document)
	exclusive: [..._allMetadataNames] | *null
}

// Metadata fields that are optional
#metadataStruct: {
	[_allMetadataNames]: #metadataField
}
_metadata: #metadataStruct & {
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
			IF `ver` does not == `id` then a document with 
			`id` and `ver` being equal *MUST* exist.
			"""
	}
	// Document Version
	ver: {
		required: "yes"
		format:   "Document Ver"
		description: """
			The unique version of the document.
			The first version of the document must set `ver` == `id`
			"""

		validation: """
			The document version must always be >= the document ID.
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
			that is required to retrieve it.  Therefore it is required that Document References
			are unique and reproducible, given a documents contents.
			"""
		validation: """
			The following must be true for a valid reference:

			* The Referenced Document **MUST** Exist
			* Every value in the `document_locator` must consistently reference the exact same document.
			* The `document_id` and `document_ver` **MUST** match the values in the referenced document.
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
			However, any document submission referencing a proposal MUST be signed by all collaborators in
			addition to the author.
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

			In this case, when the latest document is revoked, the payload may be empty.
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

	brand_id: {
		description: "A reference to the Brand Parameters Document this document lies under."
		validation: """
			In addition to the validation performed for `Document Reference` type fields: 

			* Any linked referenced document that includes a `brand_id` must match the 
			`brand_id` of the referencing document.
			"""
		exclusive: [
			"campaign_id",
			"category_id",
		]
	}

	campaign_id: {
		description: "A reference to the Campaign Parameters Document this document lies under."
		validation: """
			In addition to the validation performed for `Document Reference` type fields: 

			* Any linked referenced document that includes a `campaign_id` must match the 
			`campaign_id` of the referencing document.
			"""
		exclusive: [
			"brand_id",
			"category_id",
		]
	}

	election_id: {
		description: "A reference to the Election Parameters Document this document lies under."
		validation: """
			In addition to the validation performed for `Document Reference` type fields, 
			Any referenced document that includes a `election_id` must match the 
			`election_id` of the referencing document.
			It is also valid for the referenced document to not include this field, if it is 
			optional for the referenced document.
			"""
	}

	category_id: {
		description: "A reference to the Category Parameters Document this document lies under."
		validation: """
			In addition to the validation performed for `Document Reference` type fields: 

			* Any linked referenced document that includes a `category_id` must match the 
			`category_id` of the referencing document.
			"""
		exclusive: [
			"brand_id",
			"campaign_id",
		]
	}

}

// Note: we make all normally excluded fields optional at the global level, because they are globally optional
metadata: _metadata
metadata: {
	ref: required:           "optional"
	ref: type:               _allDocNamesList
	template: required:      "optional"
	template: type:          #templateDocNamesList
	reply: required:         "optional"
	reply: type:             #commentDocNamesList
	section: required:       "optional"
	collaborators: required: "optional"
	brand_id: required:      "optional"
	brand_id: type:          "Brand Parameters"
	campaign_id: required:   "optional"
	campaign_id: type:       "Campaign Parameters"
	election_id: required:   "optional"
	election_id: type:       "Election Parameters"
	category_id: required:   "optional"
	category_id: type:       "Category Parameters"
}

// Preferred display order
// If metadata field not listed, then list them after the explicit ones, in alphabetical order.
metadata_order: list.UniqueItems
metadata_order: [..._allMetadataNames] & [
	"type",
	"id",
	"ver",
	"ref",
	"template",
	"reply",
	"section",
	"collaborators",
	"revocations",
	"brand_id",
	"campaign_id",
	"category_id",
	"election_id",
]
