// Signed Document Definitions
// 
// COSE Signed Document CDDL Definitions
package signed_docs

cddlDefinitions: #cddlDefinitions & {
	"signed_document": {
		def: "COSE_Sign"
		requires: ["COSE_Sign"]
		description: """
			Catalyst Signed Document.

			A specific implementation of a COSE-SIGN data objects
			used by the Catalyst project to encapsulate and authenticate
			documents used within the system.

			See: \(documentationLinks."RFC9052-CoseSign")
			"""
		comment: """
			Catalyst Signed Document data object.
			"""
	}
	"COSE_Sign": {
		def: """
			[
				COSE_Document_Headers,
				payload : bstr / nil,
				signatures : [+ COSE_Signature]
			]		
			"""
		requires: ["COSE_Document_Headers", "COSE_Signature"]
		comment: "COSE-SIGN data object"
	}
	"COSE_Document_Headers": {
		def: """
			(
				protected : bstr .cbor COSE_Document_Header_Map,
				unprotected : {} ; Unused and ignored
			)
			"""
		requires: ["COSE_Document_Header_Map"]
		comment: "COSE Document headers (only protected headers are used)"
	}
	"COSE_Document_Header_Map": {
		def: """
			{
				COSE_Document_Standard_Headers,
				* label => values		
			}
			"""
		requires: ["COSE_Document_Standard_Headers"]
		comment: "COSE Document Header Map"
	}
	"COSE_Document_Standard_Headers": {
		def:     """
			(
				\(cose_headers."content type".coseLabel) => \(_cddlContentTypes),  ; content type
				?"\(cose_headers."content-encoding".coseLabel)" => [ *tstr ],  ; content encoding
			)
			"""
		comment: "COSE Standard headers used by a Document"
	}
	"COSE_Signature_Headers": {
		def: """
			(
				protected : empty_or_serialized_map,
				unprotected : {} ; Unused
			)
			"""
		requires: ["COSE_empty_or_serialized_map", "COSE_header_map"]
		comment: "COSE Document Signature headers (only protected headers are used)"
	}
	"COSE_empty_or_serialized_map": {
		def: "bstr .cbor COSE_header_map / bstr .size 0"
		requires: ["COSE_header_map"]
	}
	"COSE_header_map": {
		def: """
			{
				COSE_Generic_Headers,
				* label => values		
			}
			"""
		requires: ["COSE_Generic_Headers"]
	}
	"COSE_empty_or_serialized_map": {
		def: "bstr .cbor COSE_header_map / bstr .size 0"
		requires: ["COSE_header_map"]
	}
	"COSE_Signature": {
		def: """
			[
				COSE_Headers,
				signature : bstr
			]
			"""
		requires: ["COSE_Signature_Headers"]
	}
	"COSE_Generic_Headers": {
		def: """
			(
				; ? 1 => int / tstr,  ; algorithm identifier
				; ? 2 => [+label],    ; criticality
				? 3 => tstr / int,  ; content type
				; ? 4 => bstr,        ; key identifier
				; ? 5 => bstr,        ; IV
				; ? 6 => bstr,        ; Partial IV
				; ? 7 => COSE_Signature / [+COSE_Signature] ; Counter signature
			)

			"""
	}
}
