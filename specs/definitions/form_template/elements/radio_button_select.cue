// Radio Button selection from a list of text options.

package form_template

import (
	"github.com/input-output-hk/catalyst-libs/specs/regex"

)

dictionary: radioButtonSelect: {
	description: """
		## Functional Behavior

		Select one option from a list of items.
		Only one choice is allowed.

		## Visual Representation

		A list of items from which one item can be selected.
		The selector is styled as a set of Radio Buttons.
		"""
	parent: ["section"]

	definition: {
		type:             "string"
		contentMediaType: "text/plain"
		pattern:          regex.def.singleLine.pattern
	}
	parameters: {
		title: example: "Radio Selector"
		description: example: """
			Choose your favorite radio station.
			"""
		enum: {
			description: """
				Sorted array of string values from which a single value can be selected.
				Values must be presented in the order they appear in the array.
				No value that is not in the array may be listed or presented.
				Each item in the array **MUST** be  unique.
				"""
			items: contentMediaType: definition.contentMediaType
			example: [
				"Hot FM",
				"AM Stereo (but not really)",
				"Silence",
			]
		}
		default: {
			description:      "The option from the **`enum`** which is chosen by default.<br>This **MUST** be a value defined in the **`enum`**."
			example:          "\(enum.example[2])"
			required:         "yes"
			contentMediaType: definition.contentMediaType
		}
		"x-guidance": example: """
			Video killed the radio star.
			"""
		"x-icon": example: "bottom-rail-toggle"
	}
}
