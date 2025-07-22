// Single Line Text Entry List

package form_template

dictionary: singleLineHttpsUrlEntryList: {
	description: """
		## Functional Behavior

		A growable list of single line plain text entry fields.
		Each entry:
		
		* only supports plain text. (No markup or line breaks).
		* Can only accept text which starts with `https://` and 
		is followed by one or more non-whitespace characters,
		ending at the end of the string.
		* *MUST* be a `\(definition._itemElement)` type Form element
		and can be parameterized in the same way as that Element type.

		## Visual Representation

		Preferably, A minimum of one (and maximum of `maxItems`) 
		`https://` text entry boxes are presented.
		
		The user can complete the entry as they would a single line plain text entry field.
		They may choose to add another `https://` text entry to the list,
		or remove an existing one.

		The user should be able to insert a line in any position, and delete
		any line already entered.

		The values they enter are encoded in the order they appear on screen, 
		in the order they appear in the array.

		The Items should appear and be parameterizable in the same way the
		base `\(definition._itemElement)` type Form element can be.

		Each entry ***MUST*** be unique.
		"""
	parent: ["section"]

	definition: {
		type:         "array"
		_itemElement: "singleLineHttpsUrlEntry"
		items: $ref: "#/$defs/\(_itemElement)"
		uniqueItems: true
	}
	parameters: {
		title: example:       "Development Domains"
		description: example: "All the development domains used by your project."
		default: {
			description: "Default Array of URLs can be supplied."
			required:    "optional"
		}
		minItems: example:     0
		maxItems: example:     10
		"x-guidance": example: "Don't list production domains."
		"x-placeholder": example: ["Url 1", "Url 2", "Url 3"]
		"x-icon": example: "dots-vertical"
	}
}
