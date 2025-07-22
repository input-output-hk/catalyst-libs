// Single Line Text Entry
package form_template

import (
	"github.com/input-output-hk/catalyst-libs/specs/regex"

)

dictionary: multiLineTextEntryMarkdown: {
	description: """
		## Functional Behavior

		A multi line text entry field, with Markdown format.
		Line breaks, and special characters are allowed.
		Special formatted markup is supported.

		## Visual Representation

		A Richtext entry box that allows multiple lines of 
		formatted text up to the maximum number of 
		allowed characters.
		The character limit is defined by the total number of
		characters including markup, not the raw text itself.

		eg: `## A subtitle` is counted as 13 characters, not 10.
		"""
	definition: {
		type:             "string"
		contentMediaType: "text/markdown"
		pattern:          regex.def.multiLine.pattern
	}
	parent: ["section"]
	parameters: {
		title: example:       "Story"
		description: example: "Tell a story to the reader."
		minLength: example:   20
		maxLength: example:   5000
		default: {
			description: """
				The default value to be used if the field is empty.
				Takes priority over `x-placeholder` if both are defined.
				Allows for Markdown formatted text, like the field itself.
				"""
			required: "optional"
			example: """
				# My Story
				
				Once **upon** a *time*...
				"""
		}
		"x-guidance": example: """
			Engaging stories are better than boring ones.
			Try to be engaging.
			"""
		"x-placeholder": example: "# ..."
		"x-icon": example:        "book-open"
	}
}
