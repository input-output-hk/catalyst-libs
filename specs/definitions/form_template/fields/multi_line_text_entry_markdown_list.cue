// Single Line Text Entry List

package form_template

dictionary: multiLineTextEntryListMarkdown: {
	description: """
		UI - A Growable List of markdown formatted text fields.
		Each entry is a multiline markdown formatted string.
		Markdown Formatting, line breaks, or special characters are allowed.
		Add multiple text entries.
		Each entry should be unique.
		"""
	definition: {
		type:  "array"
		items: dictionary.multiLineTextEntryMarkdown.definition
		items: minLength: 1
		uniqueItems: true
	}
	parent: "section"
	parameters: {
		title: {}
		description: {}
		default: {
			description: "Default Array of text can be supplied."
			required:    "optional"
		}
		minItems: {}
		maxItems: {}
		contains: {}
		"x-guidance": {}
		"x-placeholder": {}
	}
}
