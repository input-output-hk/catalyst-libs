// Single Line Text Entry
package template_field_definitions

dictionary: #jsonSchemaFields & {
	singleLineTextEntry: {
		description: """
			UI - Single Line text entry without any markup or rich text capability.
			A single line of text.
			No formatting, markup, line breaks, or special characters are allowed.
			"""
		definition: {
			type:             "string"
			contentMediaType: "text/plain"
			pattern:          _regexTextSingleLine
		}
		parameters: {
			title: {}
			description: {}
			minLength: {}
			maxLength: {}
			"x-guidance": {}
			"x-placeholder": {}
		}
	}
}
