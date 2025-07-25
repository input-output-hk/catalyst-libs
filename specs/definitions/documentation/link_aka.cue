package documentation

import (
	"list"
)

#linkAKAName: string

linkAKA: [#linkAKAName]: #allLinkNames

// Known aliases for links.  Lets us automatically create [Named Link][Reference Link]
linkAKA: {
	UUIDv7:                                 "RFC9562-V7"
	UUIDv4:                                 "RFC9562-V4"
	UUID:                                   "RFC9562"
	URI:                                    "RFC3986"
	"UTF-8":                                "RFC3629"
	"CBOR Encoded IPLD Content Identifier": "CBOR-TAG-42"
	"IPFS CID":                             "IPFS-CID"
	"CBOR Deterministic Encoding":          "CBOR-LFD-ENCODING"
	BROTLI:                                 "RFC7932"
	JSON:                                   "RFC8259"
	CDDL:                                   "RFC8610"
	CBOR:                                   "RFC8949"
	COSE:                                   "RFC9052"
	"COSE Sign":                            "RFC9052-CoseSign"
	"COSE Header Parameters":               "RFC9052-HeaderParameters"
	"RFC9165 - CDDL Additional Controls":   "RFC9165"
	"text/plain;":                          "text/plain"
	"text/markdown;":                       "CommonMark"
	"text/html;":                           "HTML5"
	"text/css;":                            "text/css"
	"charset=utf-8":                        "RFC3629"
	"charset=utf-8;":                       "RFC3629"
	"template=handlebars":                  "Handlebars"
	"application/schema+json":              "JSON Schema-2020-12"
	"application/cddl":                     "RFC8610"
	"JSON Schema Draft 2020-12":            "JSON Schema-2020-12"
	Markdown:                               "CommonMark"
	HTML:                                   "HTML5"
	"JSON Schema":                          "JSON Schema-2020-12"
	"JSON Schema version 2020-12":          "JSON Schema-2020-12"
}

all_aka_names: list.UniqueItems
all_aka_names: list.Sort([
	for k, _ in linkAKA {k},
], list.Ascending)

// Reports incompatible list length if we have a link name match an aka name.
aka_not_in_names: []
aka_not_in_names: [
	for n in all_aka_names if list.Contains(all_link_names, n) {n},
]

all_link_and_aka_names: list.UniqueItems
all_link_and_aka_names: list.Sort(
	list.Concat(
	[all_link_names, all_aka_names],
	), list.Ascending)

#allLinkAndAKANames: or(all_link_and_aka_names)
