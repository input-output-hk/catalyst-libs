// Single Line Text Entry List

package form_template

dictionary: singleLineTextEntryList: {
	description: """
		UI - A Growable List of single line text
		A single line of text.
		No formatting, markup, line breaks, or special characters are allowed.
		Add multiple single-line text entries.
		Each entry should be unique.
		"""
	parent: "section"

	definition: {
		type:  "array"
		items: dictionary.singleLineTextEntry.definition
		items: minLength: 1
		uniqueItems: true
	}
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
