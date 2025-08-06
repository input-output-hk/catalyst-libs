// Single Line HTTPS URL Text Entry
package form_template

import (
	"github.com/input-output-hk/catalyst-libs/specs/regex"

)

dictionary: singleLineHttpsUrlEntry: {
	description: """
		## Functional Behavior

		A single line plain text entry field.
		Can only accept text which starts with `https://` and 
		is followed by one or more non-whitespace characters,
		ending at the end of the string.

		Must be a validly formatted https URL.

		## Visual Representation

		A Text entry box that allows a single line of plain text
		up to the maximum number of allowed characters.
		Can automatically provide `https://` to help end user
		enter the data.
		"""
	parent: ["section"]

	definition: {
		type:             "string"
		format:           "uri"
		contentMediaType: "text/plain"
		pattern:          regex.def.httpsUrl.pattern
	}
	parameters: {
		title: example:       "Website"
		description: example: "Whats your companies primary URL."
		minLength: example:   12
		maxLength: example:   1024
		default: {
			description: "The value given if nothing is entered."
			required:    "optional"
		}
		"x-guidance": example:    "Its where people find your company online."
		"x-placeholder": example: "https://<your website>"
		"x-icon": example:        "globe-alt"
	}
}
