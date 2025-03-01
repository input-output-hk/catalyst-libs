// Signed Document Definitions
// 
// COSE Headers and Constraints
package signed_docs

import (
	"list"
	"github.com/input-output-hk/catalyst-libs/specs/generic:optional"

)

// Supported Content Types (list of values)
#contentType:
	*"application/json" |
	"application/schema+json" |
	"application/cbor" |
	"application/cddl"

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

#coseHeaderFormat:
	"COSE Algorithm" |
	"IANA Media Type" |
	"HTTP Content Encoding"

#coseField: {
	coseLabel:   int | string
	description: string
	required:    optional.#field | *"yes"
	format:      #coseHeaderFormat
	if format == "IANA Media Type" {
		"value": #contentType
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
	// Default Signature Algorithm
	alg: #coseField & {
		coseLabel:   1
		required:    "optional"
		format:      "COSE Algorithm"
		description: "Default cryptographic signature algorithm"
	}

	// Documents content type
	"content type": #coseField & {
		coseLabel:   3
		format:      "IANA Media Type"
		description: "IANA Media Type/s allowed in the Payload"
	}
	// Allowed content encodings
	"content-encoding": #coseField & {
		coseLabel:   "content-encoding"
		format:      "HTTP Content Encoding"
		required:    "optional"
		description: "Supported HTTP Encodings of the Payload"
	}
}
