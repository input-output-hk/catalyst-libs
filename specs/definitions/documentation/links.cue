// External Documentation related definitions
// Links
package documentation

import (
	"list"
	"strings"
	"github.com/input-output-hk/catalyst-libs/specs/generic:url"
)

// A named Link to an external document, this would be encoded into markdown as:
// [name]: url
#docLinks: [string]: url.#absHttpsUrl

links: #docLinks & {
	"application/json":                    "https://www.iana.org/assignments/media-types/application/json"
	"application/cbor":                    "https://www.iana.org/assignments/media-types/application/cbor"
	br:                                    "https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Encoding#br"
	"JSON Schema-draft7":                  "https://json-schema.org/draft-07"
	"JSON Schema-2020-12":                 "https://json-schema.org/draft/2020-12"
	RFC7932:                               "https://www.rfc-editor.org/rfc/rfc7932"                                         // Brotli
	RFC8259:                               "https://www.rfc-editor.org/rfc/rfc8259.html"                                    // JSON
	RFC8610:                               "https://www.rfc-editor.org/rfc/rfc8610"                                         // CDDL
	RFC8949:                               "https://www.rfc-editor.org/rfc/rfc8949.html"                                    // CBOR
	RFC9052:                               "https://datatracker.ietf.org/doc/html/rfc9052"                                  // COSE
	"RFC9052-CoseSign":                    "https://datatracker.ietf.org/doc/html/rfc9052#name-signing-with-one-or-more-si" // COSE Multiple Signers
	"RFC9052-HeaderParameters":            "https://www.rfc-editor.org/rfc/rfc8152#section-3.1"                             // COSE Header Parameters
	RFC9165:                               "https://www.rfc-editor.org/rfc/rfc9165"                                         // CDDL Additional Controls
	CommonMark:                            "https://spec.commonmark.org/0.31.2/"
	RFC3629:                               "https://datatracker.ietf.org/doc/html/rfc3629"                     // UTF-8
	RFC3986:                               "https://datatracker.ietf.org/doc/html/rfc3986"                     // URI
	RFC9562:                               "https://www.rfc-editor.org/rfc/rfc9562.html"                       // UUID
	"RFC9562-V4":                          "https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-4"   // UUID V4
	"RFC9562-V7":                          "https://www.rfc-editor.org/rfc/rfc9562.html#name-uuid-version-7"   // UUID V7
	"CC-BY-4.0":                           "https://creativecommons.org/licenses/by/4.0/legalcode"             // CC BY 4.0
	"IPFS-CID":                            "https://docs.ipfs.tech/concepts/content-addressing/#what-is-a-cid" // IPFS Content Identifier
	"CBOR-TAG-42":                         "https://github.com/ipld/cid-cbor/"                                 // IPLD content identifiers (CIDs) in CBOR
	"CBOR-TAG-37":                         "https://github.com/lucas-clemente/cbor-specs/blob/master/uuid.md"  // UUID Tag for CBOR
	"CBOR-LFD-ENCODING":                   "https://www.rfc-editor.org/rfc/rfc8949.html#section-4.2.3"         // CBOR length-first core deterministic encoding requirements
	Handlebars:                            "https://handlebarsjs.com/"
	Mustache:                              "https://mustache.github.io/mustache.5.html"
	HTML5:                                 "https://html.spec.whatwg.org/multipage/syntax.html#syntax"
	CSS:                                   "https://www.w3.org/Style/CSS/"
	"text/plain":                          "https://www.rfc-editor.org/rfc/rfc2046.html"
	"text/css":                            "https://www.rfc-editor.org/rfc/rfc2318.html"
	RFC6901:                               "https://datatracker.ietf.org/doc/html/rfc6901"
	"CIP-1852":                            "https://cips.cardano.org/cip/CIP-1852"
	"historical dates":                    "https://www.oxfordreference.com/display/10.1093/acref/9780191737152.timeline.0001"
	"BLAKE2b-256":                         "https://www.blake2.net/blake2.pdf"
	"BLAKE2b-512":                         "https://www.blake2.net/blake2.pdf"
	ristretto255:                          "https://ristretto.group"
	"treasury system paper":               "https://eprint.iacr.org/2018/435.pdf"
	"treasury system specification":       "https://github.com/input-output-hk/treasury-crypto/blob/master/docs/voting_protocol_spec/Treasury_voting_protocol_spec.pdf"
	"Understanding Cryptography Textbook": "https://gnanavelrec.wordpress.com/wp-content/uploads/2019/06/2.understanding-cryptography-by-christof-paar-.pdf"
}

// Constrains the URLs being linked to be unique
#uniqueLinkValues: list.UniqueItems
#uniqueLinkValues: [...url.#absHttpsUrl] & [
	for _, v in links {v},
]

all_links: list.UniqueItems
all_links: [...url.#absHttpsUrl] & list.Sort([
	for _, v in links {v},
], list.Ascending)

#allLinks: or(all_links)

#linkName: string
#linkName: strings.MinRunes(2)

all_link_names: list.UniqueItems
all_link_names: [...#linkName] & list.Sort([
	for k, _ in links {k},
], list.Ascending)

#allLinkNames: or(all_link_names)
