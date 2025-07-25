// Single Line Text Entry List

package form_template

dictionary: singleLineTextEntryList: {
	description: """
		## Functional Behavior

		A growable list of single line plain text entry fields.
		Each entry:
		
		* only supports plain text. (No markup or line breaks).
		* *MUST* be a `\(definition._itemElement)` type Form element
		and can be parameterized in the same way as that Element type.

		## Visual Representation

		Preferably, A minimum of one (and maximum of `maxItems`) 
		plain text entry boxes are presented.
		
		The user can complete the entry as they would a single line plain text entry field.
		They may choose to add another single line text entry to the list,
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
		_itemElement: "singleLineTextEntry"
		items: $ref: "#/$defs/\(_itemElement)"
		uniqueItems: true
	}
	parameters: {
		title: example:       "Nicknames"
		description: example: "All your nicknames."
		default: {
			description: "Default Array of text can be supplied."
			required:    "optional"
			example: ["Me", "You", "Hey"]
		}
		minItems: example:     2
		maxItems: example:     17
		"x-guidance": example: "All the nicknames people have used to refer to you.\nMake some up if you have less than 2."
		"x-placeholder": example: ["nickname1", "nickname2"]
		"x-icon": example: "clipboard-list"
	}
}
