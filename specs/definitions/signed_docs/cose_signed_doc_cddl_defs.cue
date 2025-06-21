// Signed Document Definitions
// 
// COSE Signed Document CDDL Definitions
package signed_docs

import (
	"strings"
	"github.com/input-output-hk/catalyst-libs/specs/media_types"
)

// Formatted content strings to use in CDDL Definition.
_cddlContentTypes: "\"\(strings.Join(cose.headers."content type".value, "\" /\n        \""))\""

// Formatted CoAP content string to use in CDDL Definition.
_cddlCoapTypes: "\(strings.Join(media_types.allCoapTypesStr, " / "))"

cddlDefinitions: #cddlDefinitions & {
	signed_document: {
		requires: ["COSE_Sign"]
		def:         "\(requires[0])"
		description: """
			Catalyst Signed Document.

			A specific implementation of a COSE-SIGN data objects
			used by the Catalyst project to encapsulate and authenticate
			documents used within the system.

			See: \(documentation.links."RFC9052-CoseSign")
			"""
		comment: """
			Catalyst Signed Document data object.
			"""
	}
	COSE_Sign: {
		requires: [
			"COSE_Document_Headers",
			"COSE_Signature",
		]
		def:     """
			[
			  \(requires[0]),
			  payload : bstr / nil,
			  signatures : [+ \(requires[1])]
			]		
			"""
		comment: "COSE-SIGN data object"
	}
	COSE_Document_Headers: {
		requires: [
			"COSE_Document_Header_Map",
			"COSE_Generic_Headers",
		]
		def:     """
			(
			  protected   : bstr .cbor \(requires[0]),
			  unprotected : { \(requires[1]) } ; Unused and ignored
			)
			"""
		comment: "COSE Document headers (only protected headers are used)"
	}
	COSE_Document_Header_Map: {
		requires: [
			"COSE_Document_Standard_Headers",
			"Signed_Document_Metadata_Headers",
			"COSE_Generic_Headers",
		]
		def:     """
			{
			  \(requires[0]),
			  \(requires[1]),
			  \(requires[2])
			}
			"""
		comment: "COSE Document Header Map"
	}
	COSE_Document_Standard_Headers: {
		def:     """
			(
				? 1 => int / tstr,  ; algorithm identifier
				? 2 => [+\(requires[0])],    ; criticality
				? 3 => tstr / int,  ; content type
				? 4 => bstr,        ; key identifier
				? ( 5 => bstr //    ; IV
					6 => bstr )     ; Partial IV
			)		
			"""
		comment: "COSE Standard headers used by a Document"
		requires: [
			"COSE_label",
		]
	}
	Signed_Document_Metadata_Headers: {
		def:     "\(requires[0])"
		comment: "Generic definition (does not include metadata constraints)"
		requires: [
			"COSE_Generic_Headers",
		]
	}
	COSE_Signature_Headers: {
		requires: [
			"COSE_Signature_Header_Map",
			"COSE_Generic_Headers",
		]
		def:     """
			(
			  protected   : bstr .cbor \(requires[0]),
			  unprotected : { \(requires[1]) } ; Unused and ignored
			)
			"""
		comment: "COSE Signature headers (only protected headers are used)"
	}
	COSE_Signature: {
		def: """
			[
			  \(requires[0]),
			  signature : bstr
			]
			"""
		requires: ["COSE_Signature_Headers"]
		comment: "An Individual Document Signature"
	}
	COSE_Signature_Header_Map: {
		requires: [
			"COSE_Signature_Standard_Headers",
			"COSE_Generic_Headers",
		]
		def:     """
			{
			  \(requires[0]),
			  \(requires[1])
			}
			"""
		comment: "COSE Signature Header Map"
	}
	COSE_Signature_Standard_Headers: {
		def:     """
			(
				? 1 => int / tstr,  ; algorithm identifier
				? 2 => [+\(requires[0])],    ; criticality
				? 3 => tstr / int,  ; content type
				? 4 => bstr,        ; key identifier
				? ( 5 => bstr //    ; IV
					6 => bstr )     ; Partial IV
			),
			\(requires[1])
			
			"""
		comment: "COSE Signature headers"
		requires: [
			"COSE_label",
			"COSE_Generic_Headers",
		]
	}
	COSE_Generic_Headers: {
		def:     "( * \(requires[0]) => \(requires[1]) )"
		comment: "Generic Header definition"
		requires: [
			"COSE_label",
			"COSE_values",
		]
	}
	COSE_label: {
		def:     "int / tstr"
		comment: "COSE Map Generic Label"
	}
	COSE_values: {
		def:     "any"
		comment: "COSE Map Generic Value"
	}
}
