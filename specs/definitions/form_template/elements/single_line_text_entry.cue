// Single Line Text Entry
package form_template

import (
	"github.com/input-output-hk/catalyst-libs/specs/regex"

)

dictionary: singleLineTextEntry: {
	description: """
		## Functional Behavior

		A single line plain text entry field.
		Special characters are allowed.
		Line breaks, and Special formatted markup, such as Markdown are not allowed.

		## Visual Representation

		A Text entry box that allows a single line of plain text
		up to the maximum number of allowed characters.
		"""
	parent: ["section"]

	definition: {
		type:             "string"
		contentMediaType: "text/plain"
		pattern:          regex.def.singleLine.pattern
	}
	parameters: {
		title: example:       "First Name"
		description: example: "Whats your first name."
		minLength: example:   2
		maxLength: example:   300
		default: {
			description: "The value given if nothing is entered."
			example:     "Rocket"
			required:    "optional"
		}
		"x-guidance": example:    "Its the thing your parents called you."
		"x-placeholder": example: "???"
		"x-icon": example:        "user"
	}
}
