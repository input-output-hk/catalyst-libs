// Single Line Text Entry
package form_template

dictionary: #formTemplate & {
	multiLineTextEntryMarkdown: {
		description: """
			UI - Multiline text entry with Markdown content.
			Use Markdown formatting for rich text. 
			Markdown formatting is as defined by <https://spec.commonmark.org/0.31.2/>

			The following Markdown Extensions are also supported:

			* None
			"""
		definition: {
			type:             "string"
			contentMediaType: "text/markdown"
			pattern:          _regexTextMultiLine
		}
		parent: "section"
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
