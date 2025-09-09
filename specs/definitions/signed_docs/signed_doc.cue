// Signed Document Definitions
// 
// Base Types and Constraints
package signed_docs

import (
	"github.com/input-output-hk/catalyst-libs/specs/generic:uuid"
	"github.com/input-output-hk/catalyst-libs/specs/form_template/elements:form_template"
	"github.com/input-output-hk/catalyst-libs/specs/presentation_template/definedCards:presentation_template"
	"github.com/input-output-hk/catalyst-libs/specs/signed_doc_types"
)

// Document Type must be a valid UUIDv4
#docType: uuid.#v4

// Document ID or Version must be a valid UUIDv7
#docIdOrVer: uuid.#v7

// Individual Signed Document Definition
#signedDocument: {
	// If `true` the document not fully defined and still under the active development
	draft: bool | *false

	// The Document Type Identifier
	type!: #docType

	// The description of this document.  Markdown Supported.
	description?: string

	// The description of this document.  Markdown Supported.
	validation?: string

	// The business logic related to this document.  Markdown Supported.
	business_logic?: {
		front_end?: string
		back_end?:  string
	}

	notes: [...string] | *[]

	if payload.nil {
		headers: "content type": required:     "excluded"
		headers: "content-encoding": required: "excluded"
	}

	headers: _coseHeaders

	// The Metadata fields in this document (non cose standard)
	metadata: #metadata

	// Requirements for the document payload.
	payload?: #payload

	// Required/Allowed Signers of a document
	signers: _allowedSigners

	// Authors which worked on a specific document.
	authors: #authorList

	// Change Log for every doc
	versions: [#changelogEntry, ...#changelogEntry]
}

// We can only define known documents in the document definitions object
#DocumentDefinitions: {
	[signed_doc_types.#allDocNames]: #signedDocument
}

// Default Definitions for all documents.
// Customize each document type in its own `<document_name>.cue` file.
docs: #DocumentDefinitions & {
	for k, v in signed_doc_types.allDocTypes {
		(k): type: v
	}
}

doc_clusters: signed_doc_types.doc_clusters

// base Document Types to help with automated processing of the document schema information.
//base_types: _allDocTypes

formTemplate: {
	elements: form_template.dictionary
	schema:   form_template.formTemplate
	assets: icons: form_template.allIconsSvg
}

presentationTemplate: {
	cards:  presentation_template.allCards
	schema: presentation_template.presentationTemplate
}
