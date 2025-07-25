// Single Line Text Entry
package form_template

import (
	"github.com/input-output-hk/catalyst-libs/specs/regex"

)

dictionary: multiLineTextEntry: {
	description: """
		## Functional Behavior

		A multi line plain text entry field.
		Line breaks, and special characters are allowed.
		Special formatted markup, such as Markdown are not allowed.

		## Visual Representation

		A Text entry box that allows multiple lines of plain text
		up to the maximum number of allowed characters.
		"""
	parent: ["section"]

	definition: {
		type:             "string"
		contentMediaType: "text/plain"
		pattern:          regex.def.multiLine.pattern
	}
	parameters: {
		title: example:       "Explanation"
		description: example: "Explain what it is you want to achieve."
		minLength: example:   20
		maxLength: example:   5000
		default: {
			description: """
				The default value to be used if the field is empty.
				Takes priority over `x-placeholder` if both are defined.
				"""
			required: "optional"
			example:  "This explanation has not been given.\nIgnore it."
		}
		"x-guidance": example:    "It's useful to explain things here."
		"x-placeholder": example: "What's your explanation?"
		"x-icon": example:        "academic-cap"
	}
}
