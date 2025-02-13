// Signed Document Definitions
// 
// Base Types and Constraints
package signed_docs

import "list"

#optionalField:
	"yes" |
	"optional" |
	*"excluded"

// Supported Content Types (list of values)
#contentType:
	*"application/json" |
	"application/schema+json" |
	"application/cbor" |
	"application/cddl"

// Format of a Metadata Field
#metadataFormat:
	*"UUIDv7" |
	"Document Reference" |
	"Document Hash"

// Canonical List of COSE header names
_coseHeaderNames: list.UniqueItems
_coseHeaderNames: [
	"alg",
	"crit",
	"content type",
	"content-encoding", // Not strictly a true Cose Header, but we include it because of its relationship to `content type`
	"kid",
	"IV",
	"Partial IV",
	"counter signature",
]

#coseHeaderFormat:
	"COSE Algorithm" |
	"IANA Media Type" |
	"HTTP Content Encoding"

// Canonical List of all valid metadata names
_metadataNames: list.UniqueItems
_metadataNames: [
	"id",
	"ver",
	"ref",
	"ref_hash",
	"template",
	"reply",
]

// Supported content encodings (list of values)
// All documents support content encoding, this defines the supported encoding types.
// Documents may also not encode data, and will omit this field.
#contentEncoding: ["br"]

#contentEncodings: [...#contentEncoding]

// A UUIDv4 formatted string regex
#uuidv4: =~"^[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-4[0-9A-Fa-f]{3}-[89ABab][0-9A-Fa-f]{3}-[0-9A-Fa-f]{12}$"

// A uuidv7 formatted string regex
#uuidv7: =~"^[0-9A-Fa-f]{8}-[0-9A-Fa-f]{7}-4[0-9A-Fa-f]{3}-[89ABab][0-9A-Fa-f]{3}-[0-9A-Fa-f]{12}$"

// Document Type must be a valid UUIDv4
#docType: #uuidv4

// Document ID or Version must be a valid UUIDv7
#docIdOrVer: #uuidv7

// Definition of a metadata field.
#metadataField: {
	// Format of the field.
	format: #metadataFormat
	// Is the field required to be present.
	required: #optionalField
	if format == "Document Reference" {
		// If the field is Type of Document Reference, what document/s does it refer to.
		"ref type": #DocumentName | [#DocumentName, #DocumentName, ...#DocumentName]
	}

	// Optional markdown description of the field.
	description?: string
	// Optional notes about validating the field.
	validation?: string

	if format == "Template" {
		// What the template_ref must point to in the template for it to be valid.
		template_ref: #DocumentName
	}
}

#coseField: {
	coseLabel:   int | string
	description: "Default cryptographic algorithm to use"
	required:    #optionalField | *"yes"
	format:      #coseHeaderFormat
	if format == "IANA Media Types" {
		"value": "IANA"
		//"value": {
		//	[...#contentType] | *[#contentType]
		//}
	}
	if format == "HTTP Content Encoding" {
		"value": #contentEncoding
	}

}

// Metadata Fields that are required for every signed document
#coseHeaders: {
	[_coseHeaderNames]: #coseField
}
#coseHeaders: {
	// Default Signature Algorithm
	alg: {
		coseLabel: 1
		required:  "optional"
		format:    "COSE Algorithm"
	}

	// Documents content type
	"content type": {
		coseLabel: 3
		format:    "IANA Media Type"
	}
	// Allowed content encodings
	"content-encoding": {
		coseLabel: "content-encoding"
		format:    "HTTP Content Encoding"
		required:  "optional"
	}
}

// Metadata fields that are optional
#metadataStruct: {
	[_metadataNames]: #metadataField
}
#metadata: #metadataStruct & {
	// Document ID
	id: #metadataField & {
		required: "yes"
	}
	// Document Version
	ver: {
		required: "yes"
	}

	ref?: {
		format: "Document Reference"
	}
	//	"ref_hash"?: string``
	template?: {
		format: "Document Reference"
	}

}

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
	headers: #coseHeaders

	// The Metadata fields in this document (non cose standard)
	metadata: #metadata

	//if type == "Template" {
	// The Metadata fields in this document (non cose standard)
	//metadata: #templateMetadata
	//}
}

// Ensure that all Document IDs are Unique.
// See: all_docs.cue for a list of all known document types.
_allTypes: list.UniqueItems
_allTypes: [...#docType] & [
	for _, v in _allDocs {v},
]

// List of all the document names we have defined.
_allDocNames: or([
	for k, _ in _allDocs {k},
])

// We can only define known documents in the document definitions object
#DocumentDefinitions: {
	[_allDocNames]: #signedDocument
}

// Individual Valid Document Name constraint.
#DocumentName: _allDocNames

// Default Definitions for all documents.
// Customize each document type in its own `<document_name>.cue` file.
docs: #DocumentDefinitions & {
	for k, v in _allDocs {
		(k): {
			type: v
		}
	}
}
