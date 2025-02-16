// Signed Document Definitions
// 
// Base Types and Constraints
package signed_docs

#optionalField:
	"yes" |
	"optional" |
	*"excluded"

// A UUIDv4 formatted string regex
#uuidv4: =~"^[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-4[0-9A-Fa-f]{3}-[89ABab][0-9A-Fa-f]{3}-[0-9A-Fa-f]{12}$"

// A uuidv7 formatted string regex
#uuidv7: =~"^[0-9A-Fa-f]{8}-[0-9A-Fa-f]{7}-4[0-9A-Fa-f]{3}-[89ABab][0-9A-Fa-f]{3}-[0-9A-Fa-f]{12}$"

// Document Type must be a valid UUIDv4
#docType: #uuidv4

// Document ID or Version must be a valid UUIDv7
#docIdOrVer: #uuidv7

#templateMetadata: {
	[_metadataNames]: #metadataField
}
#templateMetadata: {
	"template doc": {
		format:   "Document Reference"
		required: "yes"
		description: """
			Metadata only in Template documents.
			Defines what type of document may use this template.
			"""
	}

	"template ref": {
		format:   "Document Reference"
		required: "yes"
		description: """
			Metadata only in Template documents.
			Defines what the `ref` field of the document using the template must be.
			Prevents a Document using the wrong kind of template.
			"""
	}

}

// Individual Signed Document Definition
#signedDocument: {
	// The Document Type Identifier
	type!: #docType

	// The description of this document.  Markdown Supported.
	description?: string

	// Fixed headers in every document
	headers: _coseHeaders

	// The Metadata fields in this document (non cose standard)
	metadata: _metadata
}

// We can only define known documents in the document definitions object
#DocumentDefinitions: {
	[_allDocNames]: #signedDocument
}

// Default Definitions for all documents.
// Customize each document type in its own `<document_name>.cue` file.
docs: #DocumentDefinitions & {
	for k, v in _allDocs {
		(k): {
			type: v
		}
	}
}
