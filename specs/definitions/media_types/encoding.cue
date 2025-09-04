// Content and Encoding Types
package media_types

import (
	"list"
)

// Content Encoding Type name : Description
encodingTypes: {
	[string]: {
		description: string // description of the content type
	}
}
encodingTypes: br: description: "BROTLI Compression"

allContentEncoding: list.Sort([
	for k, _ in encodingTypes {k},
], list.Ascending)
#allContentEncodingConstraint: or(allContentEncoding)