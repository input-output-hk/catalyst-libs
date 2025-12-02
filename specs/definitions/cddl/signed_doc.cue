// Common CDDL Definitions
// 
// CDDL Definitions
package cddl

cddlDefinitions: #cddlDefinitions & {
	document_type: {
		def: "[ + \(requires[0]) ]"
		requires: ["uuid_v4"]
		description: "Unique Document Type Identifier"
		comment:     "Document Type"
	}
	document_id: {
		def: "\(requires[0])"
		requires: ["uuid_v7"]
		description: "Unique Document Identifier"
		comment:     "Document ID"
	}
	document_ver: {
		def: "\(requires[0])"
		requires: ["uuid_v7"]
		description: "Unique Chronological Document Version Identifier"
		comment:     "Document Version"
	}
	document_locator: {
		def: """
			{ 
			  \"cid\" : \(requires[0])
			}
			"""
		requires: ["cid"]
		comment: "Where a document can be located, must be a unique identifier."
	}
	document_refs: {
		def: "[ + \(requires[0]) ]"
		requires: [
			"document_ref",
		]
		comment: """
			Reference to one or more Signed Documents.
			Sorting for each array element follows the same sort order as specified for Map Keys, 
			as defined by CBOR Deterministic Encoding (4.3.2 Length-First Map Key Ordering).
			"""
	}
	document_ref: {
		def: """
			[
			  \(requires[0])
			  \(requires[1])
			  \(requires[2])
			]
			"""
		requires: [
			"document_id",
			"document_ver",
			"document_locator",
		]
		comment: "Reference to a single Signed Document"
	}
	section_ref: {
		def: "\(requires[0])"
		requires: ["json_pointer"]
		comment: "Reference to a section in a referenced document."
	}
	collaborators: {
		def: "[ + \(requires[0]) ]"
		requires: ["catalyst_id_kid"]
		comment: """
			Allowed Collaborators on the next subsequent version of a document.
			Sorting for each array element follows the same sort order as specified for Map Keys, 
			as defined by CBOR Deterministic Encoding (4.3.2 Length-First Map Key Ordering).
			"""
	}
	revocations: {
		def: "[ * \(requires[0]) ] / true "
		requires: ["document_ver"]
		comment: "List of revoked versions of this document."
	}
	http_content_encoding: {
		def: """
			tstr .eq "br"
			"""
		comment: "Supported Content Encoding Types"
	}
	catalyst_id_kid: {
		def:     "bytes"
		comment: "UTF8 Catalyst ID URI encoded as a bytes string."
	}
	revocations: {
		def: "[ * document_ver ] / true "
		requires: ["document_ver"]
	}
	chain: {
		def: "[\(requires[0]), ? \(requires[1])]"
		requires: [
			"height",
			"document_ref",
		]
		comment: """
			Reference to the previous Signed Document in a sequence.
			* `\(requires[0])` is of the CURRENT block.
			* `\(requires[1])` is *ONLY* omitted in the very first document in a sequence.
			"""
	}
	height: {
		def: "int"
		comment: """
			The consecutive sequence number of the current document 
			in the chain.
			The very first document in a sequence is numbered `0` and it
			*MUST ONLY* increment by one for each successive document in
			the sequence.

			The FINAL sequence number is encoded with the current height
			sequence value, negated. 
			
			For example the following values for height define a chain
			that has 5 documents in the sequence 0-4, the final height 
			is negated to indicate the end of the chain:
			`0, 1, 2, 3, -4`

			No subsequent document can be chained to a sequence that has
			a final chain height.
			"""
	}

}
