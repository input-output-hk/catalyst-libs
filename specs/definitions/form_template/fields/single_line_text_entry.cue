// Single Line Text Entry
package form_template

dictionary: #formTemplate & {
	singleLineTextEntry: {
		description: """
			UI - Single Line text entry without any markup or rich text capability.
			A single line of text.
			No formatting, markup, line breaks, or special characters are allowed.
			"""
		parent: "section"

		definition: {
			type:    "string"
			pattern: _regexTextSingleLine
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
