// Signed Document Definitions
// 
// COSE Headers and Constraints
package signed_docs

import (
	"list"
	"github.com/input-output-hk/catalyst-libs/specs/generic:optional"
	"github.com/input-output-hk/catalyst-libs/specs/media_types"
)

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

_allCoseHeaderNames: or(_coseHeaderNames)

cose: headerFormats: #metadataFormats & {
	"Media Type": {
		description: "A Media Type string which identifies the payload."
		cddl:        "media_type"
	}
	"HTTP Content Encoding": {
		description: "Encoding, if any, of the payload."
		cddl:        "http_content_encoding"
	}
	"Catalyst ID": {
		description: "KID (Catalyst ID URI)"
		cddl:        "catalyst_id_kid"
	}
}

// Types of a Metadata Fields
#coseHeaderTypes: [
	for k, _ in cose.headerFormats {k},
]

// Constraint of Types of Cose Header Fields
#coseHeaderTypesConstraint: or(#coseHeaderTypes)

#coseField: {
	coseLabel:   int | string
	description: string
	format:      #coseHeaderTypesConstraint
	required: optional.#field_default_yes

	if required != "excluded" {
		if format == "Media Type" {
			value: media_types.#contentType | [...media_types.#contentType]
		}

		if format == "HTTP Content Encoding" {
			value: media_types.allContentEncoding
		}
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
		description: """
			Supported HTTP Encodings of the Payload.
			If no compression or encoding is used, then this field must not be present.
			"""
	}
}

_coseSignatureHeaders: #coseHeaders & {
	// Key identifier
	kid: #coseField & {
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

cose: headers: _coseHeaders
cose: headers: "content type": value: media_types.allContentTypes

// Preferred display order of cose header fields.
// if header not listed, display after the listed fields, in alphabetical order.
cose: headersOrder: list.UniqueItems
cose: headersOrder: [
	"content type",
	"content-encoding",
]

// Headers defined for signatures.
cose: signature_headers: _coseSignatureHeaders
