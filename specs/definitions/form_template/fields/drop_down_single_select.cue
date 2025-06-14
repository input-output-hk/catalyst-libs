// Radio Button selection from a list of text options.

package form_template

dictionary: #formTemplate & {
	dropDownSingleSelect: {
		description: """
			UI - Drop Down Selection of a single entry from the defined enum.

			Select one option from a selector styled as a dropdown menu.
			Only one choice is allowed.
			"""
		parent: "section"
		definition: {
			type:             "string"
			contentMediaType: "text/plain"
			pattern:          _regexTextSingleLine
		}
		parameters: {
			title: {}
			description: {}
			enum: {}
			"x-guidance": {}
		}
	}
}
