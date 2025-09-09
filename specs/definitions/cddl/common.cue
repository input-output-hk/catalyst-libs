// Common CDDL Definitions
// 
// CDDL Definitions
package cddl

import (
	docs "github.com/input-output-hk/catalyst-libs/specs/documentation"
	"github.com/input-output-hk/catalyst-libs/specs/media_types"
	"strings"
)

// Formatted content strings to use in CDDL Definition.
cddlContentTypes: "\"\(strings.Join(media_types.allContentTypes, "\" /\n        \""))\""

// Formatted CoAP content string to use in CDDL Definition.
cddlCoapTypes: "\(strings.Join(media_types.allCoapTypesStr, " / "))"

// Documentation links we embed inside CDDL descriptions.
documentation: links: docs.links

cddlDefinitions: #cddlDefinitions & {
	uuid_v7: {
		def:         "#6.37(bytes .size 16)"
		description: """
			Version 7 UUID
			See: \(documentation.links."RFC9562-V7")
			     \(documentation.links."CBOR-TAG-37")
			"""
		comment:     "UUIDv7"
	}
	uuid_v4: {
		def:         "#6.37(bytes .size 16)"
		description: """
			Version 4 UUID
			See: \(documentation.links."RFC9562-V4")
			     \(documentation.links."CBOR-TAG-37")
			"""
		comment:     "UUIDv4"
	}
	blake2b_256: {
		def:         "bytes .size 32"
		description: "Blake2b Hash (256 bits)"
		comment:     "Blake2B-256"
	}
	cid: {
		def:         "#6.42(bytes)"
		description: """
			IPLD content identifier.
			Also known as an IPFS CID
			See: \(documentation.links."IPFS-CID")
			     \(documentation.links."CBOR-TAG-42")
			"""
		comment: """
			IPLD content identifier
			TODO: add size limits if possible
			"""
	}
	json_pointer: {
		def:     "text"
		comment: """
			RFC6901 Standard JSON Pointer
			See: \(documentation.links."RFC6901")
			"""
	}
	media_type: {
		def: """
			(
			    (uint .eq (\(cddlCoapTypes))) / 
			    (tstr .eq (
			        \(cddlContentTypes)
			    ))
			)
			"""
		comment: """
			Supported Content Media Types.
			If the Media Type is supported by COAP, then the `uint` CoAP encoded
			version of the media type must be used, in preference to the string.
			"""
	}
	http_content_encoding: {
		def: """
			tstr .eq "br"
			"""
		comment: "Supported Content Encoding Types"
	}

}
