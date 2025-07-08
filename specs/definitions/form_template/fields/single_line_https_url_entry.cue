// Single Line HTTPS URL Text Entry
package form_template

import (
	"github.com/input-output-hk/catalyst-libs/specs/regex"

)

dictionary: singleLineHttpsURLEntry: {
	description: """
		UI - Single Line text entry for HTTPS Urls.
		"x-note": 
			Must start with 'https://' and is followed by one or more 
			non-whitespace characters, ending at the end of the string.
			
		"""
	parent: "section"

	definition: {
		type:    "string"
		format:  "uri"
		pattern: regex.def.httpsUrl.pattern
	}
	parameters: {
		title: {}
		description: {}
		minLength: {}
		maxLength: {}
		"x-guidance": {}
		"x-placeholder": {}
	}
}
