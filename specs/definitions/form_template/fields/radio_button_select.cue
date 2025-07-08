// Radio Button selection from a list of text options.

package form_template

import (
	"github.com/input-output-hk/catalyst-libs/specs/regex"

)

dictionary: radioButtonSelect: {
	description: """
		UI - Radio Button Selection.

		Select one option from a list of text options.
		Selector is styled as a set of Radio Buttons.
		"""
	parent: "section"

	definition: {
		type:             "string"
		contentMediaType: "text/plain"
		pattern:          regex.def.singleLine.pattern
	}
	parameters: {
		title: {}
		description: {}
		enum: {}
		"x-guidance": {}
	}
}
