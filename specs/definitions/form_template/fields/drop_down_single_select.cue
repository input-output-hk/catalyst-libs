// Radio Button selection from a list of text options.

package form_template

import (
	"github.com/input-output-hk/catalyst-libs/specs/regex"

)

dictionary: dropDownSingleSelect: {
	description: """
		UI - Drop Down Selection of a single entry from the defined enum.

		Select one option from a selector styled as a dropdown menu.
		Only one choice is allowed.
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
