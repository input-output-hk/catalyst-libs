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
	"Section Reference": {
		description: "A document section reference identifier"
		cddl:        "section_ref"
	}
	"Collaborators Reference List": {
		description: "A list of collaborators who can participate in drafting and submitting a document"
		cddl:        "collaborators"
	}
}

// Types of a Metadata Fields
#metadataTypes: [
	for k, _ in metadataFormats {k},
]

// Constraint of Types of Metadata Fields
#metadataTypesConstraint: or(#metadataTypes)

// Format of a Metadata Field
//#metadataFormat:
//	"UUIDv7" |
//	"Document Type" |
//	*"Document Reference" |
//	"Section Reference" |
//	"Collaborators Reference List"

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
		format:   "UUIDv7"
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
		format:   "UUIDv7"
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

			This is an Array of the format:
				`[[DocumentID, DocumentVer, DocumentHash],...]`

			* `DocumentID` is the UUIDv7 ID of the Document being referenced.
			* `DocumentVer` is the UUIDv7 Version of the Document being referenced.
			* `DocumentHash` is the Blake2b-256 Hash of the entire document being referenced, not just its payload.
			  It ensures that the intended referenced document is the one used, and there has been no substitution.
			  Prevents substitutions where a new document with the same Document ID and Ver might be published over an existing one.
			"""
		validation: """
			Every Reference Document **MUST** Exist, and **MUST** be a valid reference to the document.
			The calculated Hash of the Referenced Document **MUST** match the Hash in the reference. 
			"""
	}

	template: {
		description: "Reference to the template used to create and/or validate this document."
		validation: """
			In addition to the validation performed for `ref`, 
			The document payload is not valid if it does not validate completely against the referenced template.
			"""
	}

	reply: {
		description: """
			Reference to a Comment document type being referred to.
			"""
		validation: """
			In addition to the validation performed for `ref`, 
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

	brand_id: {
		description: "A reference to the Brand Parameters Document this document lies under."
		validation: """
			In addition to the validation performed for `ref`, 
			Any referenced document that includes a `brand_id` must match the `brand_id` 
			of the referencing document.
			It is also valid for the referenced document to not include this field, if it is 
			optional for the referenced document.
			"""
		exclusive: [
			"campaign_id",
			"category_id",
		]
	}

	campaign_id: {
		description: "A reference to the Campaign Parameters Document this document lies under."
		validation: """
			In addition to the validation performed for `ref`, 
			Any referenced document that includes a `campaign_id` must match the 
			`campaign_id` of the referencing document.
			It is also valid for the referenced document to not include this field, if it is 
			optional for the referenced document.
			"""
		exclusive: [
			"brand_id",
			"category_id",
		]
	}

	election_id: {
		description: "A reference to the Election Parameters Document this document lies under."
		validation: """
			In addition to the validation performed for `ref`, 
			Any referenced document that includes a `election_id` must match the 
			`election_id` of the referencing document.
			It is also valid for the referenced document to not include this field, if it is 
			optional for the referenced document.
			"""
	}

	category_id: {
		description: "A reference to the Category Parameters Document this document lies under."
		validation: """
			In addition to the validation performed for `ref`, 
			Any referenced document that includes a `category_id` must match the 
			`category_id` of the referencing document.
			It is also valid for the referenced document to not include this field, if it is 
			optional for the referenced document.
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
	"brand_id",
	"campaign_id",
	"category_id",
	"election_id",
]
