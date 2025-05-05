// Links to external documentation

package signed_docs

import (
	"list"
)

// A named Link to an external document, this would be encoded into markdown as:
// [name]: url
#metadataStruct: {
	[_allMetadataNames]: #metadataField
}

#namedLink: {
	[string]: string
}

// Constrains the URLs being linked to be unique
#uniqueLinkValues: list.UniqueItems
#uniqueLinkValues: [...string] & [
	for _, v in documentationLinks {v},
]

documentationLinks: #namedLink
documentationLinks: {
	"RFC3629":     "https://datatracker.ietf.org/doc/html/rfc3629"                     // UTF-8
	"RFC3986":     "https://datatracker.ietf.org/doc/html/rfc3986"                     // URI
	"RFC9562":     "https://www.rfc-editor.org/rfc/rfc9562.html"                       // UUID
	"RFC9562-V4":  "https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-4"   // UUID V4
	"RFC9562-V7":  "https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-7"   // UUID V7
	"CC-BY-4.0":   "https://creativecommons.org/licenses/by/4.0/legalcode"             // CC BY 4.0
	"IPFS-CID":    "https://docs.ipfs.tech/concepts/content-addressing/#what-is-a-cid" // IPFS Content Identifier
	"CBOR-TAG-42": "https://github.com/ipld/cid-cbor/"
	"CBOR-TAG-37": "https://github.com/lucas-clemente/cbor-specs/blob/master/uuid.md" // IPLD content identifiers (CIDs) in CBOR
}

#allLinkNames: or([
	for k, _ in documentationLinks {k},
])

linkAKA: {
	[string]: #allLinkNames
}

// Known aliases for links.  Lets us automatically create [Named Link][Reference Link]
linkAKA: {
	"UUIDv7":                               "RFC9562-V7"
	"UUIDv4":                               "RFC9562-V4"
	"UUID":                                 "RFC9562"
	"URI":                                  "RFC3986"
	"UTF-8":                                "RFC3629"
	"CBOR Encoded IPLD Content Identifier": "CBOR-TAG-42"
	"IPFS CID":                             "IPFS-CID"
}
