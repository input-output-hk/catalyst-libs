// Single Line Text Entry List

package form_template

dictionary: multiLineTextEntryMarkdownList: {
	description: """
		## Functional Behavior

		A growable list of multi line text entry field, with Markdown format.
		Each entry:
		
		* is a multiline markdown formatted string.
		* supports Markdown Formatting, line breaks, or special characters.
		* *MUST* be unique.
		* *MUST* be a `\(definition._itemElement)` type Form element
		and can be parameterized in the same way as that Element type.

		## Visual Representation

		Preferably, A minimum of one (and maximum of `maxItems`) rich text entry
		boxes are presented.
		
		The user can complete the entry as they would a single multi line text entry field.
		They may choose to add another multiline text entry to the list, or remove an existing one.
		The values they enter are encoded in the order they appear on screen, 
		in the order they appear in the array.

		The Items should appear and be parameterizable in the same way the
		base `\(definition._itemElement)` type Form element can be.
		"""
	definition: {
		type:         "array"
		_itemElement: "multiLineTextEntryMarkdown"
		items: $ref: "#/$defs/\(_itemElement)"
		uniqueItems: true
	}
	parent: ["section"]
	parameters: {
		title: example:       "Chapters"
		description: example: "A set of chapters used to tell your story."
		default: {
			description: "Default Array of text can be supplied."
			required:    "optional"
			example: ["""
				# Chapter 1

				## The beginning.

				Once upon a time.
				""",
				"""
					...
					""",
				"""
					# Chapter 93

					## The exciting finale.

					Maybe the real treasure was the friends we made along the way.

					***The End***

					(or is it...)
					""",
			]
		}
		items: dictionary.multiLineTextEntryMarkdown.parameters
		minItems: example:        1
		maxItems: example:        100
		"x-guidance": example:    "Tell us your never-ending story."
		"x-placeholder": example: "There is a default, so this placeholder won't show."
		"x-icon": example:        "collection"
	}
}
