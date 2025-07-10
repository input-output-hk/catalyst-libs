// Single Line Text Entry
package form_template

import (
	"github.com/input-output-hk/catalyst-libs/specs/regex"

)

dictionary: singleLineTextEntry: {
	description: """
		UI - Single Line text entry without any markup or rich text capability.
		A single line of text.
		No formatting, markup, line breaks, or special characters are allowed.
		"""
	parent: ["section"]

	definition: {
		type:    "string"
		pattern: regex.def.singleLine.pattern
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
