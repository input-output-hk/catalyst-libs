// Signed Document Definitions
// 
// Metadata Types and Constraints
package signed_docs

import "list"

// Format of a Metadata Field
#metadataFormat:
	*"UUIDv7" |
	"Document Reference" |
	"Document Hash" |
	"Section Reference" |
	"Collaborators Reference List"

// Canonical List of all valid metadata names
_metadataNames: list.UniqueItems
_metadataNames: [
	"id",
	"ver",
	"ref",
	"ref_hash",
	"ref_type",
	"template",
	"template_doc",
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
	// Format of the field.
	format: #metadataFormat
	if format == "Document Reference" {
		ref: {
			type: [#DocumentName, ...#DocumentName]
			if list.Contains(type, "Template") {
				// What the template_ref must point to in the template for it to be valid.
				template_ref?: #DocumentName
				// What media type must the template be
				template_media_type?: #contentType
			}
		}
	}

	// Is the field required to be present.
	required: #optionalField

	// Markdown description of the field.
	description: string
	// Optional notes about validating the field.
	validation?: string
}

// Metadata fields that are optional
#metadataStruct: {
	[_allMetadataNames]: #metadataField
}
_metadata: #metadataStruct & {
	// Document ID
	id: #metadataField & {
		required:    "yes"
		description: "Document ID, created the first time the document is created."
	}
	// Document Version
	ver: {
		required: "yes"
		description: """
			## Document Version

			The unique version of the document.
			The first version of the document must set `ver` == `id`
			"""

		validation: """
			The document version must always be >= the document ID.
			"""
	}

	ref?: {
		format: "Document Reference"
	}

	if ref != _|_ {
		ref_hash?: {
			format: "Document Hash"
		}
	}

	//	"ref_hash"?: string``
	template?: {
		format:      "Document Reference"
		description: "Reference to the template used to create and/or validate this document."
		validation:  "The document payload is not valid if it does not validate completely against the referenced template."
		ref: {
			type: ["Template"]
			template_ref:        "Template"
			template_media_type: "application/schema+json"
		}
	}

	template_doc?: {
		format:   "Document Reference"
		required: "yes"
		description: """
			Metadata only in Template documents.
			Defines what type of document may use this template.
			"""
		ref: {
			type:                _allDocNamesList
			template_ref:        "Template"
			template_media_type: "application/schema+json"
		}
	}

	reply?: {
		format:   "Document Reference"
		required: "optional"
		description: """
			Reference to a Comment document type being referred to.
			"""
		validation: """
			The `ref` of the `reply` document must be the same as
			the original comment document.
			"""
	}

	section?: {
		format:   "Section Reference"
		required: "optional"
		description: """
			A Reference to the original document, or the comment being replied to.
			"""
		validation: """
			For a non-reply this must be a valid section reference into the referenced document.
			For a reply, this must be a valid section reference into the comment being replied to.
			"""
	}

	collaborators?: {
		format:   "Collaborators Reference List"
		required: "optional"
		description: """
			A list of collaborators who may also publish updates to versions of this document.
			This should include all parties who have not signed this document directly.
			"""
		validation: """
			This list does not imply these collaborators have consented to collaborate, only that the author/s
			are permitting these potential collaborators to participate in the drafting and submission process.
			"""
	}

	brand_id?: {
		format:      "Document Reference"
		required:    "optional"
		description: "A reference to the Brand Parameters Document this document lies under."
	}

	campaign_id?: {
		format:      "Document Reference"
		required:    "optional"
		description: "A reference to the Campaign Parameters Document this document lies under."
	}

	election_id?: {
		format:      "Document Reference"
		required:    "optional"
		description: "A reference to the Election Parameters Document this document lies under."
	}

	category_id?: {
		format:      "Document Reference"
		required:    "optional"
		description: "A reference to the Category Parameters Document this document lies under."
	}

}
