// Radio Button selection from a list of text options.

package form_template

import (
	"github.com/input-output-hk/catalyst-libs/specs/regex"

)

dictionary: dropDownSingleSelect: {
	description: """
		## Functional Behavior

		Select one option from a selector styled as a dropdown menu.
		Only one choice is allowed.

		## Visual Representation

		A Drop Down Selection of a single entry from the defined enum values.
		"""
	parent: ["section"]
	definition: {
		type:             "string"
		contentMediaType: "text/plain"
		pattern:          regex.def.singleLine.pattern
	}
	parameters: {
		title: example: "Selector"
		description: example: """
			Drop Down Single Selector.
			Choose a value from the options presented.
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
				"option 1",
				"option 2",
				"option 3",
			]
		}
		default: {
			description:      "The option from the **`enum`** which is chosen by default.<br>This **MUST** be a value defined in the **`enum`**."
			example:          "\(enum.example[0])"
			required:         "yes"
			contentMediaType: definition.contentMediaType
		}
		"x-guidance": example: """
			It is recommended that a good choice be made.
			A bad choice could effect prospects of success.
			A good choice could improve them.
			So make a good choice.
			"""
		"x-icon": example: "emoji-happy"
	}
}
