// Signed Document Definitions
// 
// CDDL Definitions
package signed_docs

// List of cddl definitions, cddl_type_name: cddl_definition
#cddlDefinitions: {
	[string]: {
		def: string
		requires: [...#cddlTypesConstraint] | *[]
		description?: string // Description - multiline
		comment?:     string // Single line comments are displayed after a definition. Multiline comments, before.
	}
}

cddlDefinitions: #cddlDefinitions & {
	"uuid_v7": {
		def:         "#6.37(bytes .size 16)"
		description: """
			Version 7 UUID
			See: \(documentationLinks."RFC9562-V7")
			     \(documentationLinks."CBOR-TAG-37")
			"""
		comment:     "UUIDv7"
	}
	"uuid_v4": {
		def:         "#6.37(bytes .size 16)"
		description: "Version 4 UUID"
		comment:     "UUIDv4"
	}
	"document_type": {
		def: "[ 1* \(requires[0]) ]"
		requires: ["uuid_v4"]
		description: "Unique Document Type Identifier"
		comment:     "Document Type"
	}
	"blake2b_256": {
		def:         "bytes .size 32"
		description: "Blake2b Hash (256 bits)"
		comment:     "Blake2B-256"
	}
	"document_id": {
		def: "\(requires[0])"
		requires: ["uuid_v7"]
		description: "Unique Document Identifier"
		comment:     "Document ID"
	}
	"document_ver": {
		def: "\(requires[0])"
		requires: ["uuid_v7"]
		description: "Unique Chronological Document Version Identifier"
		comment:     "Document Version"
	}
	"cid": {
		def:         "#6.42(bytes)"
		description: """
			IPLD content identifier.
			Also known as an IPFS CID
			See: \(documentationLinks."IPFS-CID")
			     \(documentationLinks."CBOR-TAG-42")
			"""
		comment: """
			IPLD content identifier
			TODO: add size limits if possible
			"""
	}
	"document_locator": {
		def: """
			{ 
			  \"cid\" => \(requires[0])
			}
			"""
		requires: ["cid"]
		comment: "Where a document can be located, must be a unique identifier."
	}
	"document_ref": {
		def: """
			[ 1* [
			  \(requires[0]), 
			  \(requires[1]), 
			  \(requires[2])
			] ]
			"""
		requires: [
			"document_id",
			"document_ver",
			"document_locator",
		]
		comment: "Reference to another Signed Document"
	}
	"json_pointer": {
		def:     "text"
		comment: "RFC6901 Standard JSON Pointer"
	}
	"section_ref": {
		def: "\(requires[0])"
		requires: ["json_pointer"]
		comment: "Reference to a section in a referenced document."
	}
	"collaborators": {
		def: "[ * \(requires[0]) ]"
		requires: ["catalyst_id_kid"]
		comment: "Allowed Collaborators on the next subsequent version of a document."
	}
	"revocations": {
		def: "[ * \(requires[0]) ] / true "
		requires: ["document_ver"]
		comment: "List of revoked versions of this document."
	}
	"iana_media_types": {
		def:     """
			(
			  \(_cddlContentTypes)
			)
			"""
		comment: "Supported Content Media Types"
	}
	"http_content_encoding": {
		def: """
			"br"
			"""
		comment: "Supported Content Encoding Types"
	}
	"catalyst_id_kid": {
		def:     "bytes"
		comment: "UTF8 Catalyst ID URI encoded as a bytes string."
	}
	"revocations": {
		def: "[ * document_ver ] / true "
		requires: ["document_ver"]
	}
}

#cddlTypes: [
	for k, _ in cddlDefinitions {k},
]

#cddlTypesConstraint: or(#cddlTypes)
