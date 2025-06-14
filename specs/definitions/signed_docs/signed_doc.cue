// Signed Document Definitions
// 
// Base Types and Constraints
package signed_docs

import (
	"list"
	"strings"
	"github.com/input-output-hk/catalyst-libs/specs/generic:uuid"
	"github.com/input-output-hk/catalyst-libs/specs/form_template/fields:form_template"
)

// Document Type must be a valid UUIDv4
#docType: [...uuid.#v4]

// Document ID or Version must be a valid UUIDv7
#docIdOrVer: uuid.#v7

// Individual Signed Document Definition
#signedDocument: {
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

	// Fixed headers in every document
	headers: _coseHeaders

	// The Metadata fields in this document (non cose standard)
	metadata: _metadata

	// Requirements for the document payload.
	payload?: _payload

	// Required/Allowed Signers of a document
	signers: _allowedSigners

	// Authors which worked on a specific document.
	authors: #authorList

	// Change Log for every doc
	versions: [#changelogEntry, ...#changelogEntry]
}

// We can only define known documents in the document definitions object
#DocumentDefinitions: {
	[_allDocNames]: #signedDocument
}

// Default Definitions for all documents.
// Customize each document type in its own `<document_name>.cue` file.
docs: #DocumentDefinitions & {
	for k, v in _allDocs {
		(k): type: v
	}
}

// base Document Types to help with automated processing of the document schema information.
base_types: _allDocTypes

// Ensure that all Document Type IDs are Unique.
// See: all_docs.cue for a list of all known document types.
#allDocTypeIDs: list.UniqueItems

#allDocTypeIDs: [...uuid.#v4] & [
	for _, v in _allDocTypes {v},
]

// Ensure that all Document IDs are Unique.
// See: all_docs.cue for a list of all known document types.
_allTypes: list.UniqueItems
_allTypes: [...#docType] & [
	for _, v in _allDocs {v},
]

_allDocNamesList: [...string] & [
	for k, _ in _allDocs {k},
]

// List of all Comment Docs (not templates or actions)
#commentDocNamesList: [...string] & [
	for k, _ in _allDocs
	if strings.Contains(k, "Comment") &&
		!strings.Contains(k, "Template") &&
		!strings.Contains(k, "Action") {k},
]

// List of all Parameters Docs (not templates or actions)
#parameterDocNamesList: [...string] & [
	for k, _ in _allDocs
	if strings.Contains(k, "Parameter") &&
		!strings.Contains(k, "Template") &&
		!strings.Contains(k, "Action") {k},
]

// List of all Template Docs (not actions)
#templateDocNamesList: [...string] & [
	for k, _ in _allDocs
	if strings.Contains(k, "Template") &&
		!strings.Contains(k, "Action") {k},
]

// List of all the document names we have defined.
_allDocNames: or(_allDocNamesList)

// Individual Valid Document Name constraint.
#DocumentName: _allDocNames

formTemplate: form_template.dictionary
