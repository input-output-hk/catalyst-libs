// Single Line Text Entry
package template_field_definitions

dictionary: #jsonSchemaFields & {
	multiLineTextEntry: {
		description: """
			UI - One or more Lines of text entry.
			Line breaks, and special characters are allowed.
			Special formatted markup, such as Markdown are not allowed.
			Enter multiple lines of plain text. You can use line breaks but no special formatting.
			"""
		definition: {
			type:             "string"
			contentMediaType: "text/plain"
			pattern:          _regexTextMultiLine
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
