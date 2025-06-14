// Radio Button selection from a list of text options.

package form_template

dictionary: #formTemplate & {
	radioButtonSelect: {
		description: """
			UI - Radio Button Selection.

			Select one option from a list of text options.
			Selector is styled as a set of Radio Buttons.
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
