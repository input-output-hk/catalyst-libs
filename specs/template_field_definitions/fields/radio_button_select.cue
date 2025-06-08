// Radio Button selection from a list of text options.

package template_field_definitions

dictionary: #jsonSchemaFields & {
	radioButtonSelect: {
		description: """
			UI - Radio Button Selection.

			Select one option from a list of text options.
			Selector is styled as a set of Radio Buttons.
			"""
		definition: {
			type:             "string"
			format:           "radioButtonSelect"
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
