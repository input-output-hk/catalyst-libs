// Signed Document Definitions
// 
// COSE Headers and Constraints
package signed_docs

import (
	"list"
	"github.com/input-output-hk/catalyst-libs/specs/generic:optional"
)

// Content Type name : Description
_contentTypes: {
	[string]: {
		description: string // description of the content type
		coap_type?:  int
	}
}
_contentTypes: {
	"application/json": {
		description: "JSON Document"
		coap_type:   50
	}
	"application/schema+json": {
		description: """
			JSON Schema Draft 7 Document; Note: 
			* This is currently an unofficial media type.
			* Draft 7 is used because of its wide support by tooling.
			"""
	}
	"application/cbor": {
		description: "RFC8949 Binary CBOR Encoded Document"
		coap_type:   60
	}
	"application/cddl": {
		description: """
			CDDL Document; Note: 
			* This is an unofficial media type
			* RFC9165 Additional Control Operators for CDDL are supported.  
			* Must not have Modules, schema must be self-contained.
			"""
	}
}

contentTypes: _contentTypes

// Content Encoding Type name : Description
_encodingTypes: {
	[string]: {
		description: string // description of the content type
	}
}
_encodingTypes: {
	"br": {
		description: "BROTLI Compression"
	}
}

encodingTypes: _encodingTypes

documentationLinks: {
	"application/json":         "https://www.iana.org/assignments/media-types/application/json"
	"application/schema+json":  "https://datatracker.ietf.org/doc/draft-bhutton-json-schema/"
	"application/cbor":         "https://www.iana.org/assignments/media-types/application/cbor"
	"br":                       "https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Encoding#br"
	"JSON Schema":              "https://json-schema.org/draft-07"
	"RFC7932":                  "https://www.rfc-editor.org/rfc/rfc7932"                                         // Brotli
	"RFC8259":                  "https://www.rfc-editor.org/rfc/rfc8259.html"                                    // JSON
	"RFC8610":                  "https://www.rfc-editor.org/rfc/rfc8610"                                         // CDDL
	"RFC8949":                  "https://www.rfc-editor.org/rfc/rfc8949.html"                                    // CBOR
	"RFC9052":                  "https://datatracker.ietf.org/doc/html/rfc9052"                                  // COSE
	"RFC9052-CoseSign":         "https://datatracker.ietf.org/doc/html/rfc9052#name-signing-with-one-or-more-si" // COSE Multiple Signers
	"RFC9052-HeaderParameters": "https://www.rfc-editor.org/rfc/rfc8152#section-3.1"                             // COSE Header Parameters
	"RFC9165":                  "https://www.rfc-editor.org/rfc/rfc9165"                                         // CDDL Additional Controls
}

// Known aliases for links.  Lets us automatically create [Named Link][Reference Link]
linkAKA: {
	"BROTLI":                             "RFC7932"
	"JSON":                               "RFC8259"
	"CDDL":                               "RFC8610"
	"CBOR":                               "RFC8949"
	"COSE":                               "RFC9052"
	"COSE Sign":                          "RFC9052-CoseSign"
	"COSE Header Parameters":             "RFC9052-HeaderParameters"
	"RFC9165 - CDDL Additional Controls": "RFC9165"
}

#allContentTypes: [
	for k, _ in _contentTypes {k},
]

#contentTypesConstraint: or(#allContentTypes)

// Supported Content Types (list of values)
//#contentType: #allContentTypes | *"application/json"
#contentType: #contentTypesConstraint | *#allContentTypes[0]

// Supported content encodings (list of values)
// All documents support content encoding, this defines the supported encoding types.
// Documents may also not encode data, and will omit this field.
#contentEncoding: ["br"]

#contentEncodings: [...#contentEncoding]

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

_allCoseHeaderNames: or([
	for k in _coseHeaderNames {k},
])

coseHeaderFormats: #metadataFormats & {
	"Media Type": {
		description: "A Media Type string which identifies the payload."
		cddl:        "media_type"
	}
	"HTTP Content Encoding": {
		description: "Encoding if any on the payload."
		cddl:        "http_content_encoding"
	}
	"Catalyst ID": {
		description: "KID (Catalyst ID URI)"
		cddl:        "catalyst_id_kid"
	}
}

// Types of a Metadata Fields
#coseHeaderTypes: [
	for k, _ in coseHeaderFormats {k},
]

// Constraint of Types of Cose Header Fields
#coseHeaderTypesConstraint: or(#coseHeaderTypes)

#coseField: {
	coseLabel:   int | string
	description: string
	required:    optional.#field | *"yes"
	format:      #coseHeaderTypesConstraint
	if format == "Media Type" {
		"value": #contentType | [...#contentType]
	}

	if format == "HTTP Content Encoding" {
		"value": #contentEncoding
	}
}

// Metadata Fields that are required for every signed document
#coseHeaders: {
	[_allCoseHeaderNames]: #coseField
}
_coseHeaders: #coseHeaders & {
	// Documents content type
	"content type": #coseField & {
		coseLabel:   3
		format:      "Media Type"
		description: "Media Type/s allowed in the Payload"
	}
	// Documents Used content encodings
	"content-encoding": #coseField & {
		coseLabel: "content-encoding"
		format:    "HTTP Content Encoding"
		required:  "optional"
		description: """
			Supported HTTP Encodings of the Payload.
			If no compression or encoding is used, then this field must not be present.
			"""
	}
}

_coseSignatureHeaders: #coseHeaders & {
	// Key identifier
	"kid": #coseField & {
		coseLabel: 4
		format:    "Catalyst ID"
		description: """
			Catalyst ID URI identifying the Public Key.

			The `kid` is a UTF-8 encoded Catalyst ID URI.
			Any `kid` URI which conforms to the Catalyst ID specification may be used.
			The Catalyst ID unambiguously defines both the signing keys and signing algorithm
			used to sign the protected portion of the document.			
			"""
	}
}

cose_headers: _coseHeaders
cose_headers:
	"content type":
		value: #allContentTypes

// Preferred display order of cose header fields.
// if header not listed, display after the listed fields, in alphabetical order.
cose_headers_order: list.UniqueItems
cose_headers_order: [
	"content type",
	"content-encoding",
]

// Headers defined for signatures.
cose_signature_headers: _coseSignatureHeaders
