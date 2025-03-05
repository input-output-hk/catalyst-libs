// Signed Document Definitions
// 
// Metadata Types and Constraints
package signed_docs

import (
	"list"
	"github.com/input-output-hk/catalyst-libs/specs/generic:optional"

)

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
	format: #metadataFormat
	if format == "Document Reference" {
		type:      #DocumentName
		multiple?: bool | *false
	}

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
	ver: #metadataField & {
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

	ref?: #metadataField & {
		format: "Document Reference"
		description: """
			Reference to a Linked Document or Documents.  
			This is the primary hierarchial reference to a related document.

			This is an Array of the format:
				`[[DocumentID, DocumentVer, DocumentHash],...]`

			* `DocumentID` is the [UUIDv7] ID of the Document being referenced.
			* `DocumentVer` is the [UUIDv7] Version of the Document being referenced.
			* `DocumentHash` is the Blake2b-256 Hash of the entire document being referenced, not just its payload.
				It ensures that the intended referenced document is the one used, and there has been no substitution.
				Prevents substitutions where a new document with the same Document ID and Ver might be published over an existing one.
			"""
	}

	template?: #metadataField & {
		format:      "Document Reference"
		description: "Reference to the template used to create and/or validate this document."
		validation:  "The document payload is not valid if it does not validate completely against the referenced template."
	}

	reply?: #metadataField & {
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

	section?: #metadataField & {
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

	collaborators?: #metadataField & {
		format:   "Collaborators Reference List"
		required: "optional"
		description: """
			A list of collaborators who may also publish updates to versions of this document.
			This should include all parties who have not signed this document directly.

			Every subsequent version can amend the collaborators list.
			However, the initial Author can never be removed from being able to
			publish new version of the document.
			"""
		validation: """
			This list does not imply these collaborators have consented to collaborate, only that the author/s
			are permitting these potential collaborators to participate in the drafting and submission process.
			However any document submission referencing a proposal MUST be signed by all collaborators in
			addition to the author.
			"""
	}

	brand_id?: #metadataField & {
		format:      "Document Reference"
		description: "A reference to the Brand Parameters Document this document lies under."
		validation: """
			Any referenced document that includes a `brand_id` must match the `brand_id` 
			of the referencing document.
			It is also valid for the referenced document to not include this field, if it is 
			optional for the referenced document.
			"""
	}

	campaign_id?: #metadataField & {
		format:      "Document Reference"
		description: "A reference to the Campaign Parameters Document this document lies under."
		validation: """
			Any referenced document that includes a `campaign_id` must match the `campaign_id` 
			of the referencing document.
			It is also valid for the referenced document to not include this field, if it is 
			optional for the referenced document.
			"""
	}

	election_id?: #metadataField & {
		format:      "Document Reference"
		description: "A reference to the Election Parameters Document this document lies under."
		validation: """
			Any referenced document that includes a `election_id` must match the `election_id` 
			of the referencing document.
			It is also valid for the referenced document to not include this field, if it is 
			optional for the referenced document.
			"""
	}

	category_id?: #metadataField & {
		format:      "Document Reference"
		description: "A reference to the Category Parameters Document this document lies under."
		validation: """
			Any referenced document that includes a `category_id` must match the `category_id` 
			of the referencing document.
			It is also valid for the referenced document to not include this field, if it is 
			optional for the referenced document.
			"""
	}

}
